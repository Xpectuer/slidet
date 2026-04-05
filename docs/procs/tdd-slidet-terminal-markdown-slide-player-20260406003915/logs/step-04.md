## Step 5 — Add image capability detection and graceful fallback

### Actions Taken
- Red: added an image test around missing assets and confirmed no image helper existed yet.
- Green: implemented `ImageRender`, terminal capability detection, and fallback messages for missing files or unsupported terminals.
- Refactor: kept terminal support checks explicit and returned structured fallback text instead of panicking.

### Verify Result
- `cargo test --manifest-path slidet/Cargo.toml`
- Result: passed.
