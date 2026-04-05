#!/bin/bash
# init-proc.sh — Initialize a proc directory from a draft directory
# Usage: init-proc.sh --draft-dir <abs-path> --proc-dir <abs-path> --task-name <name>
# Usage: init-proc.sh register-proc --draft-dir <abs-path> --proc-path <repo-relative-path>
# Exit 0: fully initialized  |  Exit 1: any failure

set -euo pipefail

usage() {
  echo "Usage: init-proc.sh --draft-dir <abs-path> --proc-dir <abs-path> --task-name <name>"
  echo ""
  echo "  --draft-dir   Absolute path to the source draft directory"
  echo "  --proc-dir    Absolute path to the proc directory to create"
  echo "  --task-name   Task name (used for plan snapshot filename)"
  exit 1
}

register_usage() {
  echo "Usage: init-proc.sh register-proc --draft-dir <abs-path> --proc-path <repo-relative-path>"
  echo ""
  echo "  --draft-dir   Absolute path to the source draft directory"
  echo "  --proc-path   Repo-relative path to register in procs.txt"
  exit 1
}

normalize_proc_path() {
  local repo_root="$1"
  local proc_path="$2"

  proc_path="${proc_path#./}"
  if [[ "$proc_path" == /* ]]; then
    case "$proc_path" in
      "$repo_root"/*) printf '%s\n' "${proc_path#"$repo_root"/}" ;;
      *) echo "[ERROR] Proc path must live inside the repo: $proc_path" >&2; return 1 ;;
    esac
  else
    printf '%s\n' "$proc_path"
  fi
}

append_unique_line() {
  local file="$1"
  local line="$2"

  mkdir -p "$(dirname "$file")"
  touch "$file"
  if grep -Fxq -- "$line" "$file"; then
    return 1
  fi
  printf '%s\n' "$line" >> "$file"
}

register_proc() {
  local DRAFT_DIR=""
  local PROC_PATH=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --draft-dir) DRAFT_DIR="$2"; shift 2 ;;
      --proc-path) PROC_PATH="$2"; shift 2 ;;
      --help|-h) register_usage ;;
      *) echo "[ERROR] Unknown argument: $1" >&2; register_usage ;;
    esac
  done

  if [ -z "$DRAFT_DIR" ] || [ -z "$PROC_PATH" ]; then
    echo "[ERROR] Missing required arguments." >&2
    register_usage
  fi

  if [ ! -d "$DRAFT_DIR" ]; then
    echo "[ERROR] Draft directory does not exist: $DRAFT_DIR" >&2
    exit 1
  fi

  if [ ! -f "$DRAFT_DIR/session-log.md" ]; then
    echo "[ERROR] session-log.md not found in draft directory: $DRAFT_DIR" >&2
    exit 1
  fi

  local repo_root
  repo_root=$(git rev-parse --show-toplevel)

  local normalized_proc_path
  normalized_proc_path=$(normalize_proc_path "$repo_root" "$PROC_PATH") || exit 1

  local PROCS_FILE="$DRAFT_DIR/procs.txt"
  if append_unique_line "$PROCS_FILE" "$normalized_proc_path"; then
    echo "[OK] Proc registered: $normalized_proc_path"
  else
    echo "[OK] Proc already registered: $normalized_proc_path"
  fi
}

verify_and_fix_symlinks() {
  local ref_dir="$1"
  local draft_dir="$2"
  local draft_name
  local errors=0

  draft_name=$(basename "$draft_dir")

  if [ ! -d "$draft_dir" ]; then
    echo "[ERROR] Draft directory does not exist: $draft_dir" >&2
    exit 1
  fi

  if [ ! -d "$ref_dir" ]; then
    echo "[ERROR] Ref directory does not exist: $ref_dir" >&2
    exit 1
  fi

  for link in "$ref_dir"/*; do
    [ -e "$link" ] || [ -L "$link" ] || continue
    local fname
    fname=$(basename "$link")
    if [ -L "$link" ] && [ ! -e "$link" ]; then
      rm "$link"
      ln -s "../../../drafts/$draft_name/$fname" "$link"
      if [ -e "$link" ]; then
        echo "[FIXED] $fname"
      else
        echo "[ERROR] Could not fix symlink: $fname" >&2
        rm "$link"
        errors=$((errors + 1))
      fi
    else
      echo "[OK]    $fname"
    fi
  done

  local link_count
  local file_count
  link_count=$(find "$ref_dir" -maxdepth 1 -type l | wc -l | tr -d ' ')
  file_count=$(find "$draft_dir" -maxdepth 1 -type f | wc -l | tr -d ' ')

  if [ "$link_count" -lt "$file_count" ]; then
    for f in "$draft_dir"/*; do
      [ -f "$f" ] || continue
      local fname
      fname=$(basename "$f")
      if [ ! -e "$ref_dir/$fname" ]; then
        ln -s "../../../drafts/$draft_name/$fname" "$ref_dir/$fname"
        echo "[FIXED] $fname (missing symlink created)"
      fi
    done
    link_count=$(find "$ref_dir" -maxdepth 1 -type l | wc -l | tr -d ' ')
  fi

  if [ "$link_count" -gt "$file_count" ]; then
    echo "[WARN]  Stale symlinks: $link_count symlinks vs $file_count source files"
  fi

  echo "[OK]    Count: $link_count symlinks, $file_count source files"

  local key_files="plan.md requirements.md session-log.md"
  for kf in $key_files; do
    if [ ! -e "$ref_dir/$kf" ]; then
      if [ -f "$draft_dir/$kf" ]; then
        ln -s "../../../drafts/$draft_name/$kf" "$ref_dir/$kf"
        echo "[FIXED] $kf (key file symlink created)"
      else
        echo "[ERROR] Key file missing in source: $kf" >&2
        errors=$((errors + 1))
      fi
    fi
  done

  for link in "$ref_dir"/*; do
    [ -L "$link" ] || continue
    local target
    target=$(readlink "$link")
    if [[ "$target" = /* ]]; then
      echo "[FAIL]  Absolute symlink: $(basename "$link") -> $target" >&2
      errors=$((errors + 1))
    fi
  done

  if [ "$errors" -gt 0 ]; then
    echo "[FAIL]  $errors unfixable error(s)" >&2
    exit 1
  fi

  echo "[PASS]  All symlink checks passed"
}

DRAFT_DIR=""
PROC_DIR=""
TASK_NAME=""

if [[ "${1:-}" == "register-proc" ]]; then
  shift
  register_proc "$@"
  exit 0
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --draft-dir) DRAFT_DIR="$2"; shift 2 ;;
    --proc-dir) PROC_DIR="$2"; shift 2 ;;
    --task-name) TASK_NAME="$2"; shift 2 ;;
    --help|-h) usage ;;
    *) echo "[ERROR] Unknown argument: $1" >&2; usage ;;
  esac
done

if [ -z "$DRAFT_DIR" ] || [ -z "$PROC_DIR" ] || [ -z "$TASK_NAME" ]; then
  echo "[ERROR] Missing required arguments." >&2
  usage
fi

if [ ! -d "$DRAFT_DIR" ]; then
  echo "[ERROR] Draft directory does not exist: $DRAFT_DIR" >&2
  exit 1
fi

if [ ! -f "$DRAFT_DIR/plan.md" ]; then
  echo "[ERROR] plan.md not found in draft directory: $DRAFT_DIR" >&2
  exit 1
fi

if ! mkdir "$PROC_DIR" 2>/dev/null; then
  echo "[ERROR] Proc directory already exists or cannot be created: $PROC_DIR" >&2
  echo "        Each task must have its own unique directory." >&2
  exit 1
fi

mkdir "$PROC_DIR/findings"
cat > "$PROC_DIR/findings/README.md" << 'FINDINGS_EOF'
# Findings

This directory stores investigation results, research notes, and discoveries
made during task execution.

**Convention**: Files in this directory are **append-only**. Do not modify or
delete existing findings — add new files or append to existing ones.
FINDINGS_EOF

mkdir "$PROC_DIR/ref"

cp "$DRAFT_DIR/plan.md" "$PROC_DIR/${TASK_NAME}_plan.md"

DRAFT_NAME=$(basename "$DRAFT_DIR")
for f in "$DRAFT_DIR"/*; do
  [ -f "$f" ] || continue
  fname=$(basename "$f")
  ln -s "../../../drafts/$DRAFT_NAME/$fname" "$PROC_DIR/ref/$fname"
done

verify_and_fix_symlinks "$PROC_DIR/ref" "$DRAFT_DIR"

echo "[OK] Proc directory initialized: $PROC_DIR"
