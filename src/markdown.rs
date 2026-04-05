use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::ops::Range;

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
pub struct ListItem {
    pub checked: Option<bool>,
    pub blocks: Vec<MarkdownBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableBlock {
    pub headers: Vec<Vec<InlineSpan>>,
    pub rows: Vec<Vec<Vec<InlineSpan>>>,
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

pub fn parse_blocks(markdown: &str) -> Vec<SlideBlock> {
    let mut blocks = Vec::new();
    let mut cursor = 0;
    let mut image_src: Option<String> = None;
    let mut image_alt = String::new();

    for (event, range) in Parser::new_ext(markdown, parser_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                flush_markdown_slice(markdown, &mut blocks, cursor..range.start);
                image_src = Some(dest_url.to_string());
                image_alt.clear();
            }
            Event::End(TagEnd::Image) => {
                if let Some(src) = image_src.take() {
                    blocks.push(SlideBlock::Image {
                        alt: image_alt.trim().to_string(),
                        src,
                    });
                    image_alt.clear();
                    cursor = range.end;
                }
            }
            Event::Text(content) | Event::Code(content) => {
                if image_src.is_some() {
                    image_alt.push_str(&content);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if image_src.is_some() {
                    image_alt.push(' ');
                }
            }
            _ => {}
        }
    }

    flush_markdown_slice(markdown, &mut blocks, cursor..markdown.len());
    blocks
}

pub fn extract_headings(markdown: &str) -> Vec<String> {
    parse_markdown_blocks(markdown)
        .into_iter()
        .filter_map(|block| match block {
            MarkdownBlock::Heading { content, .. } => Some(inline_text(&content)),
            _ => None,
        })
        .collect()
}

pub fn preprocess_markdown(markdown: &str, _max_width: usize) -> String {
    parse_markdown_blocks(markdown)
        .into_iter()
        .map(block_to_plain_text)
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn parse_markdown_blocks(markdown: &str) -> Vec<MarkdownBlock> {
    let events = Parser::new_ext(markdown, parser_options()).collect::<Vec<_>>();
    let mut index = 0;
    parse_block_sequence(&events, &mut index, None)
}

fn parser_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options
}

fn flush_markdown_slice(markdown: &str, blocks: &mut Vec<SlideBlock>, range: Range<usize>) {
    if range.start >= range.end {
        return;
    }

    let slice = markdown[range].trim();
    if !slice.is_empty() {
        let parsed = parse_markdown_blocks(slice);
        if !parsed.is_empty() {
            blocks.push(SlideBlock::Markdown(parsed));
        }
    }
}

fn parse_block_sequence<'a>(
    events: &[Event<'a>],
    index: &mut usize,
    end_tag: Option<TagEnd>,
) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();

    while *index < events.len() {
        if let Some(expected) = end_tag {
            if matches_end(&events[*index], expected) {
                *index += 1;
                break;
            }
        }

        match &events[*index] {
            Event::Start(Tag::Paragraph) => {
                *index += 1;
                blocks.push(MarkdownBlock::Paragraph(parse_inline_sequence(
                    events,
                    index,
                    TagEnd::Paragraph,
                )));
            }
            Event::Start(Tag::Heading { level, .. }) => {
                let tag_level = *level;
                let level = heading_level(tag_level);
                *index += 1;
                blocks.push(MarkdownBlock::Heading {
                    level,
                    content: parse_inline_sequence(events, index, TagEnd::Heading(tag_level)),
                });
            }
            Event::Start(Tag::List(start)) => {
                let ordered_start = *start;
                *index += 1;
                let items = parse_list_items(events, index, ordered_start.is_some());
                match ordered_start {
                    Some(start) => blocks.push(MarkdownBlock::OrderedList {
                        start: start as usize,
                        items,
                    }),
                    None => blocks.push(MarkdownBlock::BulletList(items)),
                }
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                let kind = *kind;
                *index += 1;
                blocks.push(MarkdownBlock::Quote(parse_block_sequence(
                    events,
                    index,
                    Some(TagEnd::BlockQuote(kind)),
                )));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = code_block_language(kind);
                *index += 1;
                blocks.push(MarkdownBlock::CodeBlock {
                    language,
                    code: collect_code_block(events, index),
                });
            }
            Event::Start(Tag::Table(_)) => {
                *index += 1;
                blocks.push(MarkdownBlock::Table(parse_table(events, index)));
            }
            Event::Rule => {
                blocks.push(MarkdownBlock::ThematicBreak);
                *index += 1;
            }
            Event::Text(text) => {
                blocks.push(MarkdownBlock::Paragraph(vec![InlineSpan::Text(
                    text.to_string(),
                )]));
                *index += 1;
            }
            Event::SoftBreak | Event::HardBreak => {
                *index += 1;
            }
            _ => {
                *index += 1;
            }
        }
    }

    blocks
}

fn parse_list_items<'a>(
    events: &[Event<'a>],
    index: &mut usize,
    ordered: bool,
) -> Vec<ListItem> {
    let mut items = Vec::new();

    while *index < events.len() {
        match &events[*index] {
            Event::Start(Tag::Item) => {
                *index += 1;
                items.push(parse_list_item(events, index));
            }
            Event::End(TagEnd::List(is_ordered)) if *is_ordered == ordered => {
                *index += 1;
                break;
            }
            _ => {
                *index += 1;
            }
        }
    }

    items
}

fn parse_list_item<'a>(events: &[Event<'a>], index: &mut usize) -> ListItem {
    let mut checked = None;
    let mut blocks = Vec::new();

    while *index < events.len() {
        match &events[*index] {
            Event::TaskListMarker(is_checked) => {
                checked = Some(*is_checked);
                *index += 1;
            }
            Event::End(TagEnd::Item) => {
                *index += 1;
                break;
            }
            Event::Start(Tag::Paragraph) => {
                *index += 1;
                blocks.push(MarkdownBlock::Paragraph(parse_inline_sequence(
                    events,
                    index,
                    TagEnd::Paragraph,
                )));
            }
            Event::Start(Tag::List(start)) => {
                let ordered_start = *start;
                *index += 1;
                let items = parse_list_items(events, index, ordered_start.is_some());
                match ordered_start {
                    Some(start) => blocks.push(MarkdownBlock::OrderedList {
                        start: start as usize,
                        items,
                    }),
                    None => blocks.push(MarkdownBlock::BulletList(items)),
                }
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                let kind = *kind;
                *index += 1;
                blocks.push(MarkdownBlock::Quote(parse_block_sequence(
                    events,
                    index,
                    Some(TagEnd::BlockQuote(kind)),
                )));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = code_block_language(kind);
                *index += 1;
                blocks.push(MarkdownBlock::CodeBlock {
                    language,
                    code: collect_code_block(events, index),
                });
            }
            Event::Start(Tag::Table(_)) => {
                *index += 1;
                blocks.push(MarkdownBlock::Table(parse_table(events, index)));
            }
            Event::Rule => {
                blocks.push(MarkdownBlock::ThematicBreak);
                *index += 1;
            }
            Event::Text(text) => {
                blocks.push(MarkdownBlock::Paragraph(vec![InlineSpan::Text(
                    text.to_string(),
                )]));
                *index += 1;
            }
            Event::SoftBreak | Event::HardBreak => {
                *index += 1;
            }
            _ => {
                *index += 1;
            }
        }
    }

    ListItem { checked, blocks }
}

fn parse_table<'a>(events: &[Event<'a>], index: &mut usize) -> TableBlock {
    let mut headers = Vec::new();
    let mut rows = Vec::new();

    while *index < events.len() {
        match &events[*index] {
            Event::Start(Tag::TableHead) => {
                *index += 1;
                headers = parse_table_cells(events, index, TagEnd::TableHead);
            }
            Event::Start(Tag::TableRow) => {
                *index += 1;
                rows.push(parse_table_cells(events, index, TagEnd::TableRow));
            }
            Event::End(TagEnd::Table) => {
                *index += 1;
                break;
            }
            _ => {
                *index += 1;
            }
        }
    }

    TableBlock { headers, rows }
}

fn parse_table_cells<'a>(
    events: &[Event<'a>],
    index: &mut usize,
    end_tag: TagEnd,
) -> Vec<Vec<InlineSpan>> {
    let mut cells = Vec::new();

    while *index < events.len() {
        if matches_end(&events[*index], end_tag) {
            *index += 1;
            break;
        }

        match &events[*index] {
            Event::Start(Tag::TableCell) => {
                *index += 1;
                cells.push(parse_inline_sequence(events, index, TagEnd::TableCell));
            }
            _ => {
                *index += 1;
            }
        }
    }

    cells
}

fn parse_inline_sequence<'a>(
    events: &[Event<'a>],
    index: &mut usize,
    end_tag: TagEnd,
) -> Vec<InlineSpan> {
    let mut spans = Vec::new();

    while *index < events.len() {
        if matches_end(&events[*index], end_tag) {
            *index += 1;
            break;
        }

        match &events[*index] {
            Event::Text(text) => {
                push_text(&mut spans, text);
                *index += 1;
            }
            Event::Code(code) => {
                spans.push(InlineSpan::Code(code.to_string()));
                *index += 1;
            }
            Event::SoftBreak | Event::HardBreak => {
                push_text(&mut spans, " ");
                *index += 1;
            }
            Event::TaskListMarker(is_checked) => {
                push_text(&mut spans, if *is_checked { "[x] " } else { "[ ] " });
                *index += 1;
            }
            Event::Start(Tag::Emphasis) => {
                *index += 1;
                spans.push(InlineSpan::Emphasis(collect_inline_text(
                    events,
                    index,
                    TagEnd::Emphasis,
                )));
            }
            Event::Start(Tag::Strong) => {
                *index += 1;
                spans.push(InlineSpan::Strong(collect_inline_text(
                    events,
                    index,
                    TagEnd::Strong,
                )));
            }
            Event::Start(Tag::Strikethrough) => {
                *index += 1;
                spans.push(InlineSpan::Strikethrough(collect_inline_text(
                    events,
                    index,
                    TagEnd::Strikethrough,
                )));
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let destination = dest_url.to_string();
                *index += 1;
                spans.push(InlineSpan::Link {
                    label: collect_inline_text(events, index, TagEnd::Link),
                    destination,
                });
            }
            Event::Start(Tag::Image { .. }) => {
                *index += 1;
                let alt = collect_inline_text(events, index, TagEnd::Image);
                if !alt.is_empty() {
                    push_text(&mut spans, alt);
                }
            }
            _ => {
                *index += 1;
            }
        }
    }

    spans
}

fn collect_inline_text<'a>(events: &[Event<'a>], index: &mut usize, end_tag: TagEnd) -> String {
    let mut text = String::new();

    while *index < events.len() {
        if matches_end(&events[*index], end_tag) {
            *index += 1;
            break;
        }

        match &events[*index] {
            Event::Text(value) | Event::Code(value) => {
                text.push_str(value);
                *index += 1;
            }
            Event::SoftBreak | Event::HardBreak => {
                text.push(' ');
                *index += 1;
            }
            Event::TaskListMarker(is_checked) => {
                text.push_str(if *is_checked { "[x] " } else { "[ ] " });
                *index += 1;
            }
            Event::Start(Tag::Emphasis) => {
                *index += 1;
                text.push_str(&collect_inline_text(events, index, TagEnd::Emphasis));
            }
            Event::Start(Tag::Strong) => {
                *index += 1;
                text.push_str(&collect_inline_text(events, index, TagEnd::Strong));
            }
            Event::Start(Tag::Strikethrough) => {
                *index += 1;
                text.push_str(&collect_inline_text(events, index, TagEnd::Strikethrough));
            }
            Event::Start(Tag::Link { .. }) => {
                *index += 1;
                text.push_str(&collect_inline_text(events, index, TagEnd::Link));
            }
            Event::Start(Tag::Image { .. }) => {
                *index += 1;
                text.push_str(&collect_inline_text(events, index, TagEnd::Image));
            }
            _ => {
                *index += 1;
            }
        }
    }

    text.trim().to_string()
}

fn collect_code_block<'a>(events: &[Event<'a>], index: &mut usize) -> String {
    let mut code = String::new();

    while *index < events.len() {
        match &events[*index] {
            Event::End(TagEnd::CodeBlock) => {
                *index += 1;
                break;
            }
            Event::Text(text) | Event::Code(text) | Event::Html(text) => {
                code.push_str(text);
                *index += 1;
            }
            Event::SoftBreak | Event::HardBreak => {
                code.push('\n');
                *index += 1;
            }
            _ => {
                *index += 1;
            }
        }
    }

    code
}

fn code_block_language(kind: &CodeBlockKind<'_>) -> Option<String> {
    match kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(info) => info
            .split_whitespace()
            .next()
            .map(str::trim)
            .filter(|language| !language.is_empty())
            .map(str::to_string),
    }
}

fn push_text<T: AsRef<str>>(spans: &mut Vec<InlineSpan>, text: T) {
    let text = text.as_ref();
    if text.is_empty() {
        return;
    }

    match spans.last_mut() {
        Some(InlineSpan::Text(existing)) => existing.push_str(text),
        _ => spans.push(InlineSpan::Text(text.to_string())),
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn matches_end(event: &Event<'_>, end_tag: TagEnd) -> bool {
    matches!(event, Event::End(tag) if *tag == end_tag)
}

fn inline_text(spans: &[InlineSpan]) -> String {
    spans
        .iter()
        .map(|span| match span {
            InlineSpan::Text(text)
            | InlineSpan::Strong(text)
            | InlineSpan::Emphasis(text)
            | InlineSpan::Strikethrough(text)
            | InlineSpan::Code(text) => text.clone(),
            InlineSpan::Link { label, destination } => format!("{label} ({destination})"),
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn block_to_plain_text(block: MarkdownBlock) -> String {
    match block {
        MarkdownBlock::Heading { content, .. } | MarkdownBlock::Paragraph(content) => {
            inline_text(&content)
        }
        MarkdownBlock::BulletList(items) => items
            .into_iter()
            .map(list_item_to_plain_text)
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::OrderedList { start, items } => items
            .into_iter()
            .enumerate()
            .map(|(idx, item)| format!("{}. {}", start + idx, list_item_body(item)))
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::Quote(blocks) => blocks
            .into_iter()
            .map(block_to_plain_text)
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::CodeBlock { language, code } => match language {
            Some(language) => format!("```{language}\n{code}```"),
            None => format!("```\n{code}```"),
        },
        MarkdownBlock::Table(table) => table_to_plain_text(table),
        MarkdownBlock::ThematicBreak => String::from("---"),
    }
}

fn list_item_to_plain_text(item: ListItem) -> String {
    let prefix = match item.checked {
        Some(true) => "- [x] ",
        Some(false) => "- [ ] ",
        None => "- ",
    };
    format!("{prefix}{}", list_item_body(item))
}

fn list_item_body(item: ListItem) -> String {
    item.blocks
        .into_iter()
        .map(block_to_plain_text)
        .collect::<Vec<_>>()
        .join(" ")
}

fn table_to_plain_text(table: TableBlock) -> String {
    let mut rendered = vec![String::from("> [table collapsed for terminal width]")];
    let headers = table
        .headers
        .iter()
        .map(|cell| inline_text(cell))
        .collect::<Vec<_>>();

    for (row_idx, row) in table.rows.into_iter().enumerate() {
        rendered.push(String::new());
        rendered.push(format!("**Row {}**", row_idx + 1));
        for (col_idx, header) in headers.iter().enumerate() {
            let value = row
                .get(col_idx)
                .map(|cell| inline_text(cell))
                .unwrap_or_default();
            rendered.push(format!("- {header}: {value}"));
        }
    }

    rendered.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{
        parse_blocks, parse_markdown_blocks, preprocess_markdown, InlineSpan, ListItem,
        MarkdownBlock, SlideBlock, TableBlock,
    };

    #[test]
    fn parse_blocks_keeps_structured_markdown_and_images_stable() {
        let markdown =
            "# Title\n\nHello **world**.\n\n![System diagram](images/flow.png)\n\n`code`";

        let blocks = parse_blocks(markdown);

        assert_eq!(
            blocks,
            vec![
                SlideBlock::Markdown(vec![
                    MarkdownBlock::Heading {
                        level: 1,
                        content: vec![InlineSpan::Text(String::from("Title"))],
                    },
                    MarkdownBlock::Paragraph(vec![
                        InlineSpan::Text(String::from("Hello ")),
                        InlineSpan::Strong(String::from("world")),
                        InlineSpan::Text(String::from(".")),
                    ]),
                ]),
                SlideBlock::Image {
                    alt: String::from("System diagram"),
                    src: String::from("images/flow.png"),
                },
                SlideBlock::Markdown(vec![MarkdownBlock::Paragraph(vec![InlineSpan::Code(
                    String::from("code"),
                )])]),
            ]
        );
    }

    #[test]
    fn parse_markdown_blocks_supports_links_tasks_tables_and_code_blocks() {
        let markdown = "\
# Tasks\n\n\
Paragraph with [docs](https://docs.rs).\n\n\
- [x] done\n\
- [ ] pending\n\n\
| Name | Status |\n\
| --- | --- |\n\
| Parser | Active |\n\n\
```rust\n\
fn main() {}\n\
```";

        let blocks = parse_markdown_blocks(markdown);

        assert_eq!(
            blocks,
            vec![
                MarkdownBlock::Heading {
                    level: 1,
                    content: vec![InlineSpan::Text(String::from("Tasks"))],
                },
                MarkdownBlock::Paragraph(vec![
                    InlineSpan::Text(String::from("Paragraph with ")),
                    InlineSpan::Link {
                        label: String::from("docs"),
                        destination: String::from("https://docs.rs"),
                    },
                    InlineSpan::Text(String::from(".")),
                ]),
                MarkdownBlock::BulletList(vec![
                    ListItem {
                        checked: Some(true),
                        blocks: vec![MarkdownBlock::Paragraph(vec![InlineSpan::Text(
                            String::from("done"),
                        )])],
                    },
                    ListItem {
                        checked: Some(false),
                        blocks: vec![MarkdownBlock::Paragraph(vec![InlineSpan::Text(
                            String::from("pending"),
                        )])],
                    },
                ]),
                MarkdownBlock::Table(TableBlock {
                    headers: vec![
                        vec![InlineSpan::Text(String::from("Name"))],
                        vec![InlineSpan::Text(String::from("Status"))],
                    ],
                    rows: vec![vec![
                        vec![InlineSpan::Text(String::from("Parser"))],
                        vec![InlineSpan::Text(String::from("Active"))],
                    ]],
                }),
                MarkdownBlock::CodeBlock {
                    language: Some(String::from("rust")),
                    code: String::from("fn main() {}\n"),
                },
            ]
        );
    }

    #[test]
    fn preprocess_markdown_collapses_tables_and_preserves_tasks_and_links() {
        let markdown = "\
See [docs](https://docs.rs).\n\n\
- [x] shipped\n\n\
| Name | Role |\n\
| --- | --- |\n\
| Alice | Engineer |";

        let rendered = preprocess_markdown(markdown, 24);

        assert!(rendered.contains("docs (https://docs.rs)"));
        assert!(rendered.contains("- [x] shipped"));
        assert!(rendered.contains("[table collapsed for terminal width]"));
        assert!(rendered.contains("- Name: Alice"));
        assert!(rendered.contains("- Role: Engineer"));
    }

    #[test]
    fn extract_headings_returns_plain_heading_text() {
        let markdown = "# Title\n\n## Subtitle with `code`";

        let headings = super::extract_headings(markdown);

        assert_eq!(
            headings,
            vec![
                String::from("Title"),
                String::from("Subtitle with code"),
            ]
        );
    }
}
