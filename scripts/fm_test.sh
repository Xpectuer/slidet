#!/usr/bin/env bash
# fm_test.sh — Tests for fm.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FM="$SCRIPT_DIR/fm.sh"
PASS=0
FAIL=0
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

assert_eq() {
  local desc="$1" expected="$2" actual="$3"
  if [[ "$expected" == "$actual" ]]; then
    echo "  PASS: $desc"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $desc"
    echo "    expected: '$expected'"
    echo "    actual:   '$actual'"
    FAIL=$((FAIL + 1))
  fi
}

assert_exit() {
  local desc="$1" expected_exit="$2"
  shift 2
  local actual_exit=0
  "$@" >/dev/null 2>&1 || actual_exit=$?
  if [[ "$expected_exit" -eq "$actual_exit" ]]; then
    echo "  PASS: $desc"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $desc (expected exit $expected_exit, got $actual_exit)"
    FAIL=$((FAIL + 1))
  fi
}

# --- Fixtures ---

cat > "$TMPDIR/valid.md" <<'EOF'
---
title: "Test Doc"
doc_type: proc
brief: "A test document"
confidence: speculative
created: 2026-03-12
updated: 2026-03-12
revision: 1
---
# Body content
EOF

cat > "$TMPDIR/missing-field.md" <<'EOF'
---
title: "Test Doc"
doc_type: proc
created: 2026-03-12
updated: 2026-03-12
revision: 1
---
# No brief or confidence
EOF

cat > "$TMPDIR/bad-enum.md" <<'EOF'
---
title: "Test"
doc_type: invalid_type
brief: "test"
confidence: maybe
created: 2026-03-12
updated: 2026-03-12
revision: 1
---
EOF

cat > "$TMPDIR/bad-date.md" <<'EOF'
---
title: "Test"
doc_type: proc
brief: "test"
confidence: speculative
created: 2026/03/12
updated: 2026-03-12
revision: 1
---
EOF

cat > "$TMPDIR/no-frontmatter.md" <<'EOF'
# Just a regular markdown file
No frontmatter here.
EOF

# batch fixtures
mkdir -p "$TMPDIR/drafts/intake-001" "$TMPDIR/drafts/intake-002"
cat > "$TMPDIR/drafts/intake-001/session-log.md" <<'EOF'
---
title: "Session"
doc_type: proc
status: active
brief: "test1"
confidence: speculative
created: 2026-03-12
updated: 2026-03-12
revision: 1
---
EOF
cat > "$TMPDIR/drafts/intake-002/session-log.md" <<'EOF'
---
title: "Session"
doc_type: proc
status: ready
brief: "test2"
confidence: speculative
created: 2026-03-12
updated: 2026-03-12
revision: 1
---
EOF

# --- Tests ---

echo "=== fm.sh get ==="
assert_eq "get title" "Test Doc" "$(bash "$FM" get "$TMPDIR/valid.md" title)"
assert_eq "get doc_type" "proc" "$(bash "$FM" get "$TMPDIR/valid.md" doc_type)"
assert_eq "get revision" "1" "$(bash "$FM" get "$TMPDIR/valid.md" revision)"
assert_exit "get missing field exits 1" 1 bash "$FM" get "$TMPDIR/valid.md" nonexistent
assert_exit "get no frontmatter exits 1" 1 bash "$FM" get "$TMPDIR/no-frontmatter.md" title
assert_exit "get missing file exits 1" 1 bash "$FM" get "$TMPDIR/nope.md" title

echo "=== fm.sh validate ==="
assert_exit "validate valid file exits 0" 0 bash "$FM" validate "$TMPDIR/valid.md"
assert_exit "validate missing field exits 1" 1 bash "$FM" validate "$TMPDIR/missing-field.md"
assert_exit "validate bad enum exits 1" 1 bash "$FM" validate "$TMPDIR/bad-enum.md"
assert_exit "validate bad date exits 1" 1 bash "$FM" validate "$TMPDIR/bad-date.md"
assert_exit "validate no frontmatter exits 1" 1 bash "$FM" validate "$TMPDIR/no-frontmatter.md"

echo "=== fm.sh validate --stdin ==="
assert_exit "validate --stdin valid" 0 bash -c "cat '$TMPDIR/valid.md' | bash '$FM' validate --stdin"
assert_exit "validate --stdin invalid" 1 bash -c "cat '$TMPDIR/missing-field.md' | bash '$FM' validate --stdin"

echo "=== fm.sh batch ==="
BATCH_OUT=$(bash "$FM" batch "$TMPDIR/drafts/" status)
assert_eq "batch line 1" "intake-001:active" "$(echo "$BATCH_OUT" | head -1)"
assert_eq "batch line 2" "intake-002:ready" "$(echo "$BATCH_OUT" | tail -1)"
assert_exit "batch missing dir exits 1" 1 bash "$FM" batch "$TMPDIR/empty-nope/" status

# ---------------------------------------------------------------------------
# Real docs integration tests (only run when docs/ exists at repo root)
# ---------------------------------------------------------------------------

GIT_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
DOCS="$GIT_ROOT/docs"

if [[ -d "$DOCS/drafts" && -d "$DOCS/procs" ]]; then

  echo "=== fm.sh get (real docs) ==="

  # Pick first draft session-log with frontmatter
  REAL_SESSION=""
  for d in "$DOCS"/drafts/intake-*/session-log.md; do
    [[ -f "$d" ]] || continue
    REAL_SESSION="$d"
    break
  done
  if [[ -n "$REAL_SESSION" ]]; then
    # Every session-log.md must have doc_type and status
    assert_eq "real get doc_type" "proc" "$(bash "$FM" get "$REAL_SESSION" doc_type)"
    REAL_STATUS=$(bash "$FM" get "$REAL_SESSION" status 2>/dev/null) || REAL_STATUS=""
    case "$REAL_STATUS" in
      active|ready|activated|abandoned)
        echo "  PASS: real get status is valid enum ($REAL_STATUS)"
        PASS=$((PASS + 1))
        ;;
      *)
        echo "  FAIL: real get status unexpected: '$REAL_STATUS'"
        FAIL=$((FAIL + 1))
        ;;
    esac
  fi

  # Pick first proc tracking file (tdd.md or progress.md)
  REAL_PROC=""
  for d in "$DOCS"/procs/*/tdd.md "$DOCS"/procs/*/progress.md; do
    [[ -f "$d" ]] || continue
    REAL_PROC="$d"
    break
  done
  if [[ -n "$REAL_PROC" ]]; then
    PROC_TITLE=$(bash "$FM" get "$REAL_PROC" title 2>/dev/null) || PROC_TITLE=""
    if [[ -n "$PROC_TITLE" ]]; then
      echo "  PASS: real proc get title is non-empty ('$PROC_TITLE')"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: real proc get title returned empty"
      FAIL=$((FAIL + 1))
    fi
  fi

  echo "=== fm.sh batch (real docs/drafts) ==="

  BATCH_DRAFTS=$(bash "$FM" batch "$DOCS/drafts/" status 2>/dev/null) || BATCH_DRAFTS=""
  BATCH_LINES=$(printf '%s\n' "$BATCH_DRAFTS" | grep -c '.' || true)
  if [[ "$BATCH_LINES" -gt 0 ]]; then
    echo "  PASS: batch docs/drafts returned $BATCH_LINES entries"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: batch docs/drafts returned 0 entries"
    FAIL=$((FAIL + 1))
  fi

  # Every batch line must match <name>:<value> format
  BAD_FORMAT=0
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if ! [[ "$line" =~ ^[^:]+:.+$ ]]; then
      BAD_FORMAT=$((BAD_FORMAT + 1))
      echo "    bad format: '$line'"
    fi
  done <<< "$BATCH_DRAFTS"
  if [[ "$BAD_FORMAT" -eq 0 ]]; then
    echo "  PASS: batch output format valid (all lines match name:value)"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: batch output has $BAD_FORMAT malformed lines"
    FAIL=$((FAIL + 1))
  fi

  echo "=== fm.sh batch (real docs/procs) ==="

  BATCH_PROCS=$(bash "$FM" batch "$DOCS/procs/" status 2>/dev/null) || BATCH_PROCS=""
  PROC_LINES=$(printf '%s\n' "$BATCH_PROCS" | grep -c '.' || true)
  if [[ "$PROC_LINES" -gt 0 ]]; then
    echo "  PASS: batch docs/procs returned $PROC_LINES entries"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: batch docs/procs returned 0 entries"
    FAIL=$((FAIL + 1))
  fi

  echo "=== fm.sh validate (real docs) ==="

  # Validate all draft session-logs; count pass/fail (don't assert all pass — some legacy docs may lack fields)
  V_PASS=0
  V_FAIL=0
  V_TOTAL=0
  for f in "$DOCS"/drafts/*/session-log.md; do
    [[ -f "$f" ]] || continue
    V_TOTAL=$((V_TOTAL + 1))
    if bash "$FM" validate "$f" 2>/dev/null; then
      V_PASS=$((V_PASS + 1))
    else
      V_FAIL=$((V_FAIL + 1))
    fi
  done
  if [[ "$V_TOTAL" -gt 0 ]]; then
    echo "  INFO: validated $V_TOTAL session-logs: $V_PASS pass, $V_FAIL fail"
    # At least some must pass (confirms validate isn't broken on real files)
    if [[ "$V_PASS" -gt 0 ]]; then
      echo "  PASS: validate works on real session-logs ($V_PASS/$V_TOTAL pass)"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: validate rejected ALL real session-logs"
      FAIL=$((FAIL + 1))
    fi
  fi

  echo "=== fm.sh validate --stdin (real docs) ==="

  # Pipe a real file through --stdin and confirm same result as file mode
  if [[ -n "$REAL_SESSION" ]]; then
    FILE_EXIT=0
    bash "$FM" validate "$REAL_SESSION" 2>/dev/null || FILE_EXIT=$?
    STDIN_EXIT=0
    cat "$REAL_SESSION" | bash "$FM" validate --stdin 2>/dev/null || STDIN_EXIT=$?
    if [[ "$FILE_EXIT" -eq "$STDIN_EXIT" ]]; then
      echo "  PASS: validate file vs --stdin agree (both exit $FILE_EXIT)"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: validate file exit=$FILE_EXIT but --stdin exit=$STDIN_EXIT"
      FAIL=$((FAIL + 1))
    fi
  fi

else
  echo "=== Skipping real docs tests (docs/drafts or docs/procs not found) ==="
fi

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[[ "$FAIL" -eq 0 ]] || exit 1
