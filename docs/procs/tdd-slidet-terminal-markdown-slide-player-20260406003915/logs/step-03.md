## Step 4 — Parse Markdown into renderable blocks

### Actions Taken
- Red: added a parser test that expects stable text and image block extraction from a mixed Markdown slide and observed the first implementation split text blocks incorrectly.
- Green: implemented `SlideBlock` plus `parse_blocks`, tracking image boundaries and producing deterministic text/image output.
- Refactor: adjusted block flushing so text remains stable across heading and paragraph boundaries until an image or document end requires a flush.

### Verify Result
- `cargo test --manifest-path slidet/Cargo.toml`
- Result: passed.
