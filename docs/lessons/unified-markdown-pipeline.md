---
title: "Lesson: Unified Markdown Pipeline Migration"
doc_type: lesson
brief: "Migrating from split markdown rendering to unified pulldown-cmark pipeline"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Lesson: Unified Markdown Pipeline Migration

## Context

`slidet` initially had a split markdown processing architecture:
- **Parsing**: `src/markdown.rs` used `pulldown-cmark` for block parsing and heading extraction
- **Rendering**: `src/ui.rs` used `tui-markdown::from_str` for text rendering

This created a **semantic drift problem**: two different markdown libraries could interpret the same markdown differently, leading to:
- Inconsistent behavior between parsing and rendering
- Higher maintenance cost (two dependencies to track)
- Unclear boundaries when extending markdown support

## Problem

The split architecture caused issues when adding support for common markdown elements:
- Which library's semantics should we follow?
- How do we ensure consistent behavior across the pipeline?
- What happens when the two libraries disagree on edge cases?

## Solution

**Unified the entire markdown pipeline around `pulldown-cmark`** by:

1. **Removed `tui-markdown` dependency** from `Cargo.toml`
2. **Introduced structured internal model** in `src/markdown.rs`:
   - `SlideBlock::Markdown(Vec<MarkdownBlock>)` instead of `SlideBlock::Text(String)`
   - `MarkdownBlock` enum: Heading, Paragraph, BulletList, OrderedList, Quote, CodeBlock, Table, ThematicBreak
   - `InlineSpan` enum: Text, Strong, Emphasis, Strikethrough, Code, Link
   - `ListItem` with task list support (`checked: Option<bool>`)
3. **Direct rendering** in `src/ui.rs`: Convert `MarkdownBlock` → `ratatui::Text` directly
4. **Expanded test coverage** for links, task lists, tables, and code blocks

## Implementation Steps

1. **Remove legacy dependency**: Delete `tui-markdown` from `Cargo.toml`
2. **Build structured model**: Replace raw string slices with AST-like block/span model
3. **Direct TUI rendering**: Convert internal model to `ratatui::Text` without intermediate library
4. **Add regression samples**: Ensure edge cases are covered (e.g., `examples/05-parser-edge-cases/04-links-and-tasks.md`)
5. **Validate**: Run `cargo check` and `cargo test` to ensure no regressions

## Code Example: Structured Model

```rust
// Before: Raw string
pub enum SlideBlock {
    Text(String),
    Image { alt: String, src: String },
}

// After: Structured model
pub enum SlideBlock {
    Markdown(Vec<MarkdownBlock>),
    Image { alt: String, src: String },
}

pub enum MarkdownBlock {
    Heading { level: u8, content: Vec<InlineSpan> },
    Paragraph(Vec<InlineSpan>),
    BulletList(Vec<ListItem>),
    OrderedList { start: usize, items: Vec<ListItem> },
    Quote(Vec<MarkdownBlock>),
    CodeBlock { language: Option<String>, code: String },
    Table(TableBlock),
    ThematicBreak,
}

pub enum InlineSpan {
    Text(String),
    Strong(String),
    Emphasis(String),
    Strikethrough(String),
    Code(String),
    Link { label: String, destination: String },
}
```

## Benefits

1. **Single source of truth**: `pulldown-cmark` is now the only markdown semantic source
2. **Easier extension**: Add new block/span types in one place (`markdown.rs`)
3. **Better testability**: Can unit test parsing independently from rendering
4. **Reduced dependencies**: One less crate to maintain and update
5. **Consistent behavior**: No risk of semantic drift between parsing and rendering

## Trade-offs

1. **More code**: Had to implement direct rendering logic in `ui.rs` instead of delegating to `tui-markdown`
2. **Manual feature parity**: Each markdown feature must be explicitly implemented in the rendering layer
3. **Maintenance burden**: Own the rendering logic instead of relying on a dedicated library

## When to Apply This Pattern

Consider unifying your markdown pipeline when:
- You're using multiple markdown libraries with overlapping responsibilities
- Semantic inconsistencies are causing bugs or user confusion
- You need fine-grained control over rendering behavior
- One of your markdown libraries is becoming a maintenance burden

## When NOT to Apply This Pattern

Keep separate libraries when:
- Each library has a distinct, non-overlapping responsibility
- The rendering library provides complex features you need (e.g., syntax highlighting with tree-sitter)
- The cost of reimplementation outweighs the benefit of unification

## Related Files

- `src/markdown.rs`: Structured markdown model and parsing logic
- `src/ui.rs`: Direct rendering of `MarkdownBlock` → `ratatui::Text`
- `examples/05-parser-edge-cases/04-links-and-tasks.md`: Regression test for links and task lists
- `docs/procs/tdd-pulldowncmark-markdown-pipeline-unification-20260406062001/`: TDD session log

## Verification

- `cargo check`: Compiles successfully without `tui-markdown`
- `cargo test`: All 18 unit tests pass (loader, markdown, image, app, ui, watcher)
- Manual testing: Common markdown elements (headings, lists, tables, links, code blocks) render correctly

## References

- TDD Session: `docs/procs/tdd-pulldowncmark-markdown-pipeline-unification-20260406062001/tdd.md`
- Requirements: `docs/drafts/intake-pulldowncmark-markdown-pipeline-unification-20260406060224/requirements.md`
- pulldown-cmark docs: https://docs.rs/pulldown-cmark/latest/pulldown_cmark/
