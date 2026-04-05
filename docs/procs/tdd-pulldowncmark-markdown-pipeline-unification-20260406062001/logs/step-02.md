## Step 2 — Replace raw markdown slices with a structured render model
### Actions Taken
- Replaced `SlideBlock::Markdown(String)` with `SlideBlock::Markdown(Vec<MarkdownBlock>)`.
- Added `MarkdownBlock`/`InlineSpan`/`ListItem`/`TableBlock` to capture headings, paragraphs, lists, quotes, fenced code, tables, links, and task states from `pulldown-cmark`.
- Reworked parsing so image events still split slide blocks while non-image markdown is parsed into the structured internal model.
- Expanded markdown unit tests to cover links, task lists, tables, fenced code blocks, and heading extraction.

### Verify Result
- Ran `cargo test markdown:: --lib`.
- Result: passed with 4 tests green after the UI layer was migrated to the new `SlideBlock::Markdown(Vec<MarkdownBlock>)` contract.
- Timestamp: 2026-04-05T22:33:55Z
