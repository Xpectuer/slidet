#!/usr/bin/env bash
# test-scan.sh — Tests for fm.sh scan subcommand
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FM="$SCRIPT_DIR/fm.sh"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

pass=0
fail=0

assert_eq() {
  local desc="$1" expected="$2" actual="$3"
  if [[ "$expected" == "$actual" ]]; then
    echo "  PASS: $desc"
    pass=$((pass + 1))
  else
    echo "  FAIL: $desc"
    echo "    expected: $expected"
    echo "    actual:   $actual"
    fail=$((fail + 1))
  fi
}

assert_exit() {
  local desc="$1" expected_code="$2"
  shift 2
  local actual_code=0
  "$@" >/dev/null 2>&1 || actual_code=$?
  assert_eq "$desc" "$expected_code" "$actual_code"
}

# Setup: drafts
mkdir -p "$TMPDIR/drafts/intake-20260101"
cat > "$TMPDIR/drafts/intake-20260101/session-log.md" <<'EOF'
---
title: "Test Draft"
doc_type: proc
status: active
brief: "Test brief text"
confidence: speculative
created: 2026-01-01
updated: 2026-01-01
revision: 1
---
# Content
EOF

mkdir -p "$TMPDIR/drafts/idea-20260102"
cat > "$TMPDIR/drafts/idea-20260102/session-log.md" <<'EOF'
---
title: "Another Draft"
doc_type: proc
status: ready
brief: "Another brief"
confidence: verified
created: 2026-01-02
updated: 2026-01-02
revision: 1
---
# Content
EOF

# Setup: procs
mkdir -p "$TMPDIR/procs/tdd-test-20260101"
cat > "$TMPDIR/procs/tdd-test-20260101/tdd.md" <<'EOF'
---
title: "TDD Proc"
doc_type: proc
status: active
source: "docs/drafts/intake-20260101"
confidence: verified
created: 2026-01-01
updated: 2026-01-01
revision: 1
---
# Steps
- [x] Step 1
- [x] Step 2
- [ ] Step 3
- [ ] Step 4
- [ ] Step 5
EOF

mkdir -p "$TMPDIR/procs/intake-symlink"
cat > "$TMPDIR/procs/intake-symlink/session-log.md" <<'EOF'
---
title: "Should be skipped"
doc_type: proc
status: active
brief: "skip me"
confidence: speculative
created: 2026-01-01
updated: 2026-01-01
revision: 1
---
EOF

# Setup: empty dir
mkdir -p "$TMPDIR/empty"

# Setup: missing field
mkdir -p "$TMPDIR/drafts/no-brief-20260103"
cat > "$TMPDIR/drafts/no-brief-20260103/session-log.md" <<'EOF'
---
title: "No Brief"
doc_type: proc
status: active
confidence: speculative
created: 2026-01-03
updated: 2026-01-03
revision: 1
---
EOF

echo "=== fm.sh scan tests ==="

# Test 1: Basic scan
echo "Test 1: Basic scan with default fields"
out=$("$FM" scan "$TMPDIR/drafts/")
line_count=$(echo "$out" | wc -l | tr -d ' ')
assert_eq "outputs 3 lines (3 drafts)" "3" "$line_count"

# Test 2: Multi-field extraction
echo "Test 2: Multi-field extraction"
out=$("$FM" scan "$TMPDIR/drafts/" --fields status,brief,updated --pattern "intake-*")
assert_eq "intake fields" "intake-20260101	active	Test brief text	2026-01-01" "$out"

# Test 3: Pattern filter
echo "Test 3: Pattern filter"
out=$("$FM" scan "$TMPDIR/drafts/" --pattern "idea-*")
line_count=$(echo "$out" | wc -l | tr -d ' ')
assert_eq "only idea dirs" "1" "$line_count"

# Test 4: Progress calculation
echo "Test 4: Progress column"
out=$("$FM" scan "$TMPDIR/procs/" --fields status,source --progress)
echo "$out" | grep -q '2/5 (40%)' && assert_eq "progress 2/5" "0" "0" || assert_eq "progress 2/5" "found" "not found"

# Test 5: Missing field outputs —
echo "Test 5: Missing field"
out=$("$FM" scan "$TMPDIR/drafts/" --fields brief --pattern "no-brief-*")
echo "$out" | grep -q '—' && assert_eq "missing brief shows —" "0" "0" || assert_eq "missing brief shows —" "found" "not found"

# Test 6: Empty directory
echo "Test 6: Empty directory exits 1"
assert_exit "empty dir exit 1" "1" "$FM" scan "$TMPDIR/empty/"

# Test 7: Header row
echo "Test 7: Header row"
out=$("$FM" scan "$TMPDIR/drafts/" --header --pattern "intake-*")
first_line=$(echo "$out" | head -1)
assert_eq "header line" "dirname	status	brief	updated" "$first_line"

# Test 8: Procs skip intake-*
echo "Test 8: Procs skip intake-*"
out=$("$FM" scan "$TMPDIR/procs/" --fields status)
echo "$out" | grep -q 'intake-symlink' && assert_eq "intake skipped" "not found" "found" || assert_eq "intake skipped" "0" "0"

echo ""
echo "=== Results: $pass passed, $fail failed ==="
[[ "$fail" -eq 0 ]] || exit 1
