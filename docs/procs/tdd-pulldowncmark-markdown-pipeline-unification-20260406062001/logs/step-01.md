## Step 1 — Remove the legacy markdown renderer dependency
### Actions Taken
- Removed `tui-markdown` from `Cargo.toml`.
- Kept the change scoped to the dependency list only.
- Noted that the worktree already contains other unrelated modifications outside this case.
### Verify Result
- Ran `rg -n "tui-markdown" Cargo.toml && exit 1 || true`.
- Result: no matches, command exited successfully.
- Timestamp: 2026-04-05T22:23:36Z
