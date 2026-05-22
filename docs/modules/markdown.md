---
doc_type: module
module_name: markdown
module_path: src/markdown.rs
generated_by: mci-phase-2
created: 2026-04-06
updated: 2026-05-23
revision: 2
brief: Markdown 解析器，将文本转换为结构化块模型（SlideBlock/MarkdownBlock/InlineSpan），支持表格折叠、标题提取和链接收集
---

# markdown 模块文档

基于 `pulldown-cmark` 的 Markdown 解析模块，将原始 Markdown 文本转换为结构化的块模型和行内元素，支持图片提取、任务列表、表格和代码块等扩展语法。

<!-- BEGIN:INTERFACE -->

## Interface

### 公开类型

| 类型 | 定义 | 说明 |
|------|------|------|
| `SlideBlock` | `enum` | 顶层块，表示一个 slide 中的独立元素 |
| `MarkdownBlock` | `enum` | Markdown 内容块，表示标题、段落、列表等结构 |
| `InlineSpan` | `enum` | 行内元素，表示文本、强调、链接等 |
| `ListItem` | `struct` | 列表项，包含可选的任务状态和嵌套块 |
| `TableBlock` | `struct` | 表格结构，包含表头和行数据 |

#### `SlideBlock` 变体

```rust
pub enum SlideBlock {
    Markdown(Vec<MarkdownBlock>),  // 结构化 Markdown 内容
    Image { alt: String, src: String },  // 独立图片块
}
```

#### `MarkdownBlock` 变体

```rust
pub enum MarkdownBlock {
    Heading { level: u8, content: Vec<InlineSpan> },  // H1-H6 标题
    Paragraph(Vec<InlineSpan>),                       // 段落
    BulletList(Vec<ListItem>),                        // 无序列表
    OrderedList { start: usize, items: Vec<ListItem> }, // 有序列表
    Quote(Vec<MarkdownBlock>),                        // 引用块
    CodeBlock { language: Option<String>, code: String }, // 代码块
    Table(TableBlock),                                // 表格
    ThematicBreak,                                    // 水平分割线
}
```

#### `InlineSpan` 变体

```rust
pub enum InlineSpan {
    Text(String),                                    // 普通文本
    Strong(String),                                  // **粗体**
    Emphasis(String),                                // *斜体*
    Strikethrough(String),                           // ~~删除线~~
    Code(String),                                    // `行内代码`
    Link { label: String, destination: String },     // [链接](url)
}
```

#### `ListItem` 结构

```rust
pub struct ListItem {
    pub checked: Option<bool>,    // 任务列表状态：Some(true/false) 或 None
    pub blocks: Vec<MarkdownBlock>, // 嵌套的内容块
}
```

#### `TableBlock` 结构

```rust
pub struct TableBlock {
    pub headers: Vec<Vec<InlineSpan>>,        // 表头单元格
    pub rows: Vec<Vec<Vec<InlineSpan>>>,      // 数据行
}
```

### 公开函数

| 函数 | 签名 | 说明 |
|------|------|------|
| `parse_blocks` | `fn(markdown: &str) -> Vec<SlideBlock>` | 主入口，解析 Markdown 为 SlideBlock 列表 |
| `parse_markdown_blocks` | `fn(markdown: &str) -> Vec<MarkdownBlock>` | 解析为 MarkdownBlock 列表（不含图片提取） |
| `extract_headings` | `fn(markdown: &str) -> Vec<String>` | 提取所有标题文本 |
| `preprocess_markdown` | `fn(markdown: &str, max_width: usize) -> String` | 预处理为纯文本，折叠表格 |
| `collect_links` | `fn(markdown: &str) -> Vec<String>` | 从所有块中收集唯一的链接 URL（去重） |

### 链接收集（内部函数）

`collect_links` 遍历解析后的 `MarkdownBlock` 树，递归收集所有 `InlineSpan::Link` 的 `destination`：

| 函数 | 签名 | 说明 |
|------|------|------|
| `collect_links` | `fn(markdown: &str) -> Vec<String>` | 公开入口，解析后去重收集链接 |
| `block_links` | `fn(block: &MarkdownBlock) -> Vec<String>` | 从单个块中提取链接（递归处理嵌套块） |
| `list_item_links` | `fn(item: &ListItem) -> Vec<String>` | 从列表项中提取链接（递归处理嵌套块） |
| `inline_links` | `fn(spans: &[InlineSpan]) -> Vec<String>` | 从行内 span 序列中提取 Link 的 destination |

覆盖的块类型：Heading, Paragraph, BulletList, OrderedList, Quote, Table（包括表头和所有行）。
不产生链接的块类型：CodeBlock, ThematicBreak（返回空 Vec）。

<!-- END:INTERFACE -->

<!-- BEGIN:DEPENDENCY_GRAPH -->

## Dependency Graph

```
markdown.rs
    |
    +-- pulldown_cmark (外部 crate)
    |       |-- Parser       : 事件流解析器
    |       |-- Event        : 解析事件枚举
    |       |-- Tag / TagEnd : 标签类型
    |       |-- HeadingLevel : 标题级别
    |       |-- CodeBlockKind: 代码块类型
    |       |-- Options      : 解析选项
    |
    +-- std::ops::Range      : 字节范围类型
```

### 外部依赖

| 依赖 | 版本约束 | 用途 |
|------|---------|------|
| `pulldown-cmark` | Cargo.toml 定义 | CommonMark 兼容的 Markdown 解析器 |

### 内部依赖

无。`markdown.rs` 是叶子模块，不依赖 `src/` 中其他模块。

### 被依赖关系

| 消费者 | 用途 |
|--------|------|
| `loader.rs` | 调用 `parse_blocks` 将 .md 文件转为 Slide |
| `ui.rs` | 使用 `SlideBlock`/`MarkdownBlock`/`InlineSpan` 类型渲染 |

<!-- END:DEPENDENCY_GRAPH -->

<!-- BEGIN:STATE_MANAGEMENT -->

## State Management

### 无状态设计

本模块的所有公开函数都是**纯函数**，不持有可变状态：

- **无全局变量**：所有解析状态通过函数参数和局部变量传递
- **无静态变量**：不使用 `static` 或 `lazy_static`
- **无内部可变性**：不使用 `RefCell`/`Mutex` 等

### 解析状态传递模式

解析过程通过**游标传递**（cursor passing）管理状态：

```rust
// 事件索引作为可变游标传入
fn parse_block_sequence(
    events: &[Event<'a>],
    index: &mut usize,      // 游标：当前事件位置
    end_tag: Option<TagEnd> // 终止条件
) -> Vec<MarkdownBlock>
```

### 临时状态变量

在 `parse_blocks` 中，图片解析使用临时状态：

```rust
let mut image_src: Option<String> = None;  // 当前图片 URL
let mut image_alt = String::new();          // 累积的 alt 文本
```

这些变量在图片标签闭合后立即消费并重置。

### 返回值所有权

所有公开函数返回**拥有所有权**的数据结构：

- `Vec<SlideBlock>` / `Vec<MarkdownBlock>` / `Vec<InlineSpan>` 由调用者拥有
- 无生命周期参数，数据可安全跨作用域传递

<!-- END:STATE_MANAGEMENT -->

<!-- BEGIN:EDGE_CASES -->

## Edge Cases

### 硬编码值

| 位置 | 值 | 说明 |
|------|-----|------|
| `parser_options()` | `ENABLE_STRIKETHROUGH` | 启用 `~~删除线~~` 语法 |
| `parser_options()` | `ENABLE_TABLES` | 启用 GFM 表格 |
| `parser_options()` | `ENABLE_TASKLISTS` | 启用 `- [ ]` / `- [x]` 任务列表 |
| `table_to_plain_text()` | `"> [table collapsed for terminal width]"` | 表格折叠占位文本 |

### 错误处理策略

本模块**不返回 Result**，采用以下降级策略：

| 场景 | 处理方式 |
|------|----------|
| 空输入 | 返回空 `Vec` |
| 无效 Markdown | 依赖 `pulldown-cmark` 的容错解析，不 panic |
| 缺少表格行 | `rows.get(col_idx).unwrap_or_default()` 返回空字符串 |
| 空 alt 文本 | `image_alt.trim().to_string()` 清理空白 |
| 空代码块语言 | `filter(|language| !language.is_empty())` 过滤 |

### 边界情况处理

1. **文本合并**：`push_text` 函数将连续的 `Text` span 合并为单个 `InlineSpan::Text`，减少碎片化

2. **嵌套解析**：列表项可包含任意 MarkdownBlock（包括嵌套列表、引用、代码块）

3. **图片内嵌**：图片出现在段落内时，alt 文本被提取为普通文本，不生成 `SlideBlock::Image`

4. **表格折叠**：`preprocess_markdown` 将表格转为带行号的多行列表格式，适配终端宽度限制

5. **任务列表标记**：在 `parse_inline_sequence` 中，`TaskListMarker` 被转换为 `[x] ` 或 `[ ] ` 文本

### 潜在问题

| 问题 | 影响 | 建议 |
|------|------|------|
| `preprocess_markdown` 的 `_max_width` 参数未使用 | 函数签名与实现不一致 | 移除参数或实现宽度适配 |
| 大文件性能 | 事件流先 collect 为 Vec | 对于超大文件考虑流式处理 |

<!-- END:EDGE_CASES -->

<!-- BEGIN:USAGE_EXAMPLE -->

## Usage Example

### 基本解析

```rust
use slidet::markdown::{parse_blocks, SlideBlock, MarkdownBlock, InlineSpan};

let markdown = r#"
# Introduction

Welcome to **slidet**!

- [x] Feature A
- [ ] Feature B

![Diagram](assets/flow.png)
"#;

let blocks = parse_blocks(markdown);

for block in blocks {
    match block {
        SlideBlock::Markdown(content) => {
            for mb in content {
                match mb {
                    MarkdownBlock::Heading { level, content } => {
                        println!("H{}: {:?}", level, content);
                    }
                    MarkdownBlock::BulletList(items) => {
                        for item in items {
                            println!("Task done: {:?}", item.checked);
                        }
                    }
                    _ => {}
                }
            }
        }
        SlideBlock::Image { alt, src } => {
            println!("Image: {} ({})", alt, src);
        }
    }
}
```

### 提取标题

```rust
use slidet::markdown::extract_headings;

let markdown = "# Title\n\n## Section\n\n### Subsection";
let headings = extract_headings(markdown);
// => ["Title", "Section", "Subsection"]
```

### 预处理为纯文本

```rust
use slidet::markdown::preprocess_markdown;

let markdown = r#"
| Name | Value |
| --- | --- |
| A | 1 |
| B | 2 |
"#;

let text = preprocess_markdown(markdown, 80);
// 包含 "> [table collapsed for terminal width]" 和展开的行数据
```

### 模式匹配行内元素

```rust
use slidet::markdown::InlineSpan;

fn render_inline(spans: &[InlineSpan]) -> String {
    spans.iter().map(|span| match span {
        InlineSpan::Text(t) => t.clone(),
        InlineSpan::Strong(t) => format!("**{}**", t),
        InlineSpan::Emphasis(t) => format!("*{}*", t),
        InlineSpan::Code(t) => format!("`{}`", t),
        InlineSpan::Link { label, destination } => format!("[{}]({})", label, destination),
        InlineSpan::Strikethrough(t) => format!("~~{}~~", t),
    }).collect()
}
```

<!-- END:USAGE_EXAMPLE -->
