## Step 3 — Implement slide directory loader

### Actions Taken
- Red: added loader-focused unit tests for filename ordering and invalid-input errors, then confirmed the loader case failed before `slidet/src/loader.rs` existed.
- Green: implemented `load_slides` and `Slide`, filtering `.md` files, sorting by filename, and returning explicit errors for missing paths, non-directories, and empty directories.
- Refactor: kept the loader self-contained and used contextual anyhow errors for directory reads and file reads.

### Verify Result
- `cargo test --manifest-path slidet/Cargo.toml loader -- --nocapture`
- Result: passed.
