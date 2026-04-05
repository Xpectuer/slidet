## Step 7 — Render browse layout and fullscreen playback

### Actions Taken
- Red: added a UI-oriented content rendering test to assert text plus fallback-image output before the rendering helper existed.
- Green: implemented terminal init/restore, browse/present rendering, slide list rendering, and content assembly from parsed Markdown blocks.
- Refactor: extracted `render_slide_content` so parsing and image fallback behavior can be tested without a live terminal frame.

### Verify Result
- `cargo test --manifest-path slidet/Cargo.toml`
- Result: passed.
- `cargo check --manifest-path slidet/Cargo.toml`
- Result: passed.
