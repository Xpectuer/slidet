## Step 1 — Create Cargo manifest

### Actions Taken
- Red: confirmed `cargo metadata --manifest-path slidet/Cargo.toml --no-deps` failed because the manifest did not exist.
- Green: created `slidet/Cargo.toml` with the package name `slidet` and the dependencies from the plan.
- Green: added a minimal `slidet/src/lib.rs` so Cargo has a target and metadata parsing can succeed.
- Refactor: kept the manifest lean and limited to the dependencies required by the plan.

### Verify Result
- `test -f slidet/Cargo.toml && rg '^name = "slidet"$' slidet/Cargo.toml && cargo metadata --manifest-path slidet/Cargo.toml --no-deps >/dev/null`
- Result: passed.
- Notes: `cargo metadata` emits a non-fatal warning unless `--format-version` is specified; the required verification command still succeeds.
