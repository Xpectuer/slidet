---
title: "Markdown Pipeline Module"
doc_type: module
brief: "Structured markdown parsing and rendering pipeline based on pulldown-cmark"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 2
---

# Markdown Pipeline Module

## Overview

The markdown pipeline provides structured parsing of Markdown content using `pulldown-cmark` as the single source of truth for Markdown semantics. It transforms raw Markdown text into a type-safe AST that can be rendered directly to `ratatui::Text`.

## Key Files

| File | Responsibility |
|------|----------------|
| `src/markdown.rs` | Parsing logic, AST definitions, table folding |
| `src/ui.rs` | Rendering `MarkdownBlock` → `ratatui::Text` |

## Data Model

### Top-Level Types

```rust
pub enum SlideBlock {
    Markdown(Vec<MarkdownBlock>),  // Structured markdown content
    Image { alt: String, src: String },  // Standalone image
}
```

### Block Types

```rust
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
```

### Inline Types

```rust
pub enum InlineSpan {
    Text(String),
    Strong(String),
    Emphasis(String),
    Strikethrough(String),
    Code(String),
    Link { label: String, destination: String },
}
```

### List Item

```rust
pub struct ListItem {
    pub checked: Option<bool>,  // Some(true) = [x], Some(false) = [ ], None = regular
    pub blocks: Vec<MarkdownBlock>,  // Supports nested content
}
```

## Supported Markdown Features

| Feature | AST Representation | Notes |
|---------|-------------------|-------|
| Headings (H1-H6) | `MarkdownBlock::Heading` | Level stored as `u8` |
| Paragraphs | `MarkdownBlock::Paragraph` | Contains `InlineSpan` sequence |
| Bullet lists | `MarkdownBlock::BulletList` | Nested lists supported |
| Ordered lists | `MarkdownBlock::OrderedList` | Start index preserved |
| Task lists | `ListItem.checked` | `[x]` / `[ ]` syntax |
| Blockquotes | `MarkdownBlock::Quote` | Can contain any block type |
| Fenced code | `MarkdownBlock::CodeBlock` | Language tag extracted |
| Tables | `MarkdownBlock::Table` | Folded to card layout in terminal |
| Horizontal rules | `MarkdownBlock::ThematicBreak` | Rendered as `---` |
| Links | `InlineSpan::Link` | Label + destination preserved |
| Images (inline) | Converted to text alt | Standalone images become `SlideBlock::Image` |
| Bold/italic/strikethrough | `InlineSpan::Strong/Emphasis/Strikethrough` | Nested spans supported |
| Inline code | `InlineSpan::Code` | Backtick-delimited |

## Parser Options

The parser enables these `pulldown-cmark` extensions:

```rust
Options::ENABLE_STRIKETHROUGH
Options::ENABLE_TABLES
Options::ENABLE_TASKLISTS
```

## Table Folding Strategy

Wide tables are collapsed to a card layout for terminal width constraints:

**Original:**
```
| Name | Role | Status |
| --- | --- | --- |
| Alice | Engineer | Active |
```

**Rendered:**
```
> [table collapsed for terminal width]

**Row 1**
- Name: Alice
- Role: Engineer
- Status: Active
```

## Public API

```rust
// Main parsing entry point
pub fn parse_blocks(markdown: &str) -> Vec<SlideBlock>

// Parse markdown content into blocks (excludes images)
pub fn parse_markdown_blocks(markdown: &str) -> Vec<MarkdownBlock>

// Extract heading text for slide titles
pub fn extract_headings(markdown: &str) -> Vec<String>

// Convert blocks to plain text (for preprocessing)
pub fn preprocess_markdown(markdown: &str, max_width: usize) -> String
```

## Rendering Contract

`src/ui.rs` converts `MarkdownBlock` directly to `ratatui::Text`:

1. `render_markdown_text(blocks: &[MarkdownBlock]) -> Text<'static>`
2. Each block type maps to styled `Line` sequences
3. Headings are center-aligned with bold style
4. Links render as `label (destination)`
5. Task items show `[x]` / `[ ]` prefixes

## Design Decision

This module was created to unify the Markdown pipeline on a single parser (`pulldown-cmark`), replacing a split architecture where `tui-markdown` was used for rendering. See `docs/lessons/unified-markdown-pipeline.md` for the rationale.
