| Step | Status | Notes |
|------|--------|-------|
| Step 1 — Create Cargo manifest | ✅ | `slidet/Cargo.toml` created; metadata verify passed after adding a minimal target stub. |
| Step 2 — Add runtime bootstrap and CLI entry | ✅ | Added `main.rs` to parse CLI args, load slides, and manage terminal lifecycle. |
| Step 3 — Implement slide directory loader | ✅ | Added loader tests plus sorted `.md` loading with explicit invalid-input errors. |
| Step 4 — Parse Markdown into renderable blocks | ✅ | Added stable text/image parsing and fixed heading/paragraph flush behavior. |
| Step 5 — Add image capability detection and graceful fallback | ✅ | Missing or unsupported images now degrade to readable placeholder text. |
| Step 6 — Implement application state and key handling | ✅ | Browse/present state transitions and core key bindings are implemented. |
| Step 7 — Render browse layout and fullscreen playback | ✅ | Browse/present UI rendering compiles and uses parsed slide content. |
