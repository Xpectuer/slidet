| Step | Status | Notes |
|------|--------|-------|
| Step 1 — Remove the legacy markdown renderer dependency | ✅ | Removed `tui-markdown` from `Cargo.toml` and verified no references remain. |
| Step 2 — Replace raw markdown slices with a structured render model | ✅ | Added structured markdown parsing model and markdown-focused tests. |
| Step 3 — Render the structured model directly in the TUI | ✅ | Replaced the old renderer with direct `MarkdownBlock` -> `ratatui::Text` conversion. |
| Step 4 — Add a regression sample for links and task lists | ✅ | Added the parser edge-case sample covering links and task items. |
| Step 5 — Validate Build and Tests | ✅ | `cargo check` and `cargo test` passed. |
