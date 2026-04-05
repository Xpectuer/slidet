## Step 2 — Add runtime bootstrap and CLI entry

### Actions Taken
- Red: confirmed the binary entrypoint did not exist, so the crate could not satisfy the runtime bootstrap case.
- Green: added `slidet/src/main.rs` to parse the slide directory argument, load slides, initialize the terminal, run the app, and restore terminal state on exit.
- Refactor: routed module access through the library crate so runtime wiring stays minimal and testable.

### Verify Result
- `cargo check --manifest-path slidet/Cargo.toml`
- Result: passed.
