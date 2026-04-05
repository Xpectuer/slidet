---
title: "Plan: Unify Markdown Pipeline on pulldown-cmark"
doc_type: proc
brief: "Implement a single pulldown-cmark-driven markdown rendering pipeline for slidet"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Plan: Unify Markdown Pipeline on pulldown-cmark

## Files Changed

| File | Change Type |
|------|-------------|
| Cargo.toml | Minor edit |
| src/markdown.rs | Major edit |
| src/ui.rs | Major edit |
| examples/05-parser-edge-cases/04-links-and-tasks.md | New file |

## Step 1 — Remove the legacy markdown renderer dependency

**File**: `Cargo.toml`  
**What**: 删除 `tui-markdown` 依赖，确保项目只保留 `pulldown-cmark` 作为 Markdown 语义来源。

**Old**:
```toml
ratatui = "0.29"
ratatui-image = "4"
ratatui-core = "0.1"
tui-markdown = { version = "0.3.7", features = ["highlight-code"] }
unicode-width = "0.2"
```

**New**:
```toml
ratatui = "0.29"
ratatui-image = "4"
ratatui-core = "0.1"
unicode-width = "0.2"
```

**Verify**: `rg -n "tui-markdown" Cargo.toml && exit 1 || true`

## Step 2 — Replace raw markdown slices with a structured render model

**File**: `src/markdown.rs`  
**What**: 把 `SlideBlock::Markdown(String)` 演进为基于 `pulldown-cmark` 事件流构建的块级 + 轻量行内片段模型，并在同文件内补齐表格降级、任务列表、链接、代码块语言标签和相关单元测试。

**Old**:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideBlock {
    Markdown(String),
    Image { alt: String, src: String },
}
```

**New**:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideBlock {
    Markdown(Vec<MarkdownBlock>),
    Image { alt: String, src: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineSpan {
    Text(String),
    Strong(String),
    Emphasis(String),
    Strikethrough(String),
    Code(String),
    Link { label: String, destination: String },
}
```

**Verify**: `cargo test markdown:: --lib`

## Step 3 — Render the structured model directly in the TUI

**File**: `src/ui.rs`  
**What**: 移除 `tui_markdown::from_str` 路径，改为把 `markdown.rs` 产出的内部模型直接转换成 `ratatui::Text`，并同步更新高度估算与 UI 测试。

**Old**:
```rust
use ratatui_image::{Resize, StatefulImage};
use tui_markdown::from_str;
```

**New**:
```rust
use ratatui_image::{Resize, StatefulImage};
```

**Old**:
```rust
fn render_markdown_block(frame: &mut Frame, area: Rect, raw_markdown: &str, text_scroll: u16) {
    let width = area.width.max(1);
    let prepared_markdown = markdown::preprocess_markdown(raw_markdown, width as usize);
    let headings = markdown::extract_headings(raw_markdown);
    let mut rendered = convert_text(from_str(&prepared_markdown));
    center_headings(&mut rendered, &headings);
```

**New**:
```rust
fn render_markdown_block(
    frame: &mut Frame,
    area: Rect,
    blocks: &[markdown::MarkdownBlock],
    text_scroll: u16,
) {
    let rendered = markdown::render_text(blocks, area.width);
```

**Verify**: `cargo test ui:: --lib`

## Step 4 — Add a regression sample for links and task lists

**File**: `examples/05-parser-edge-cases/04-links-and-tasks.md`  
**What**: 新增覆盖链接文本、URL 展示、已完成/未完成任务列表和普通段落混排的回归样例。

**Old**:
```text
[new file]
```

**New**:
```markdown
# 链接与任务列表

先看一个链接：[pulldown-cmark](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/)。

- [x] 移除分裂的 Markdown 渲染路径
- [ ] 用统一模型驱动终端展示

普通段落继续存在，用来确认任务列表之后的文本不会消失。
```

**Verify**: `rg -n "pulldown-cmark|\\[x\\]|\\[ \\]" examples/05-parser-edge-cases/04-links-and-tasks.md`

## Step 5 — Validate Build and Tests

**File**: `Cargo.toml`  
**What**: 在依赖和渲染路径改造完成后执行完整编译与测试，确认统一管线没有破坏现有行为。

**Old**:
```toml
unicode-width = "0.2"
```

**New**:
```toml
unicode-width = "0.2"
```

**Verify**: `cargo check && cargo test`

## Step 6 — Proof-Read End-to-End

Read each changed file in full. Check: formatting, no leftover TODOs, spec intent preserved.

## Step 7 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| `cargo check` 与 `cargo test` 通过。 | Step 5 |
| 示例 slides 中常见 Markdown 元素能够稳定显示。 | Step 2, Step 3, Step 4, Step 5, Step 6 |
| Markdown 显示行为围绕 `pulldown-cmark` 统一，而不是依赖分裂的多套 Markdown 语义路径。 | Step 1, Step 2, Step 3 |

## Step 8 — Review

Follow Phase 3 (see `03-self-review.md`). Writes `review.md`.

## Step 9 — Commit

Use /commit. Suggested message:
```text
feat: unify markdown rendering on pulldown-cmark
- replace tui-markdown rendering with a structured internal model
- add regression coverage for links, tasks, tables, and code blocks
```

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9
