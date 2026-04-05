## Step 6 — Implement application state and key handling

### Actions Taken
- Red: added an application-state test covering browse/present transitions, navigation, and quit handling before the app module existed.
- Green: implemented `App`, `Mode`, navigation helpers, `handle_key`, and the runtime loop that draws and consumes key events.
- Refactor: centralized scroll reset and slide access in the app state so UI rendering stays dumb.

### Verify Result
- `cargo test --manifest-path slidet/Cargo.toml`
- Result: passed.
