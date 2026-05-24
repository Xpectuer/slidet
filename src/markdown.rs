use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlideBlock {
    Markdown(Vec<MarkdownBlock>),
    Image { alt: String, src: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownBlock {
    Heading {
        level: u8,
        content: Vec<InlineSpan>,
    },
    Paragraph(Vec<InlineSpan>),
    BulletList(Vec<ListItem>),
    OrderedList {
        start: usize,
        items: Vec<ListItem>,
    },
    Quote(Vec<MarkdownBlock>),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
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
            Event::Text(content) | Event::Code(content) if image_src.is_some() => {
                image_alt.push_str(&content);
            }
            Event::SoftBreak | Event::HardBreak if image_src.is_some() => {
                image_alt.push(' ');
            }
            _ => {}
        }
    }

    flush_markdown_slice(markdown, &mut blocks, cursor..markdown.len());
    blocks
}

pub fn collect_links(markdown: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    parse_markdown_blocks(markdown)
        .iter()
        .flat_map(block_links)
        .filter(|url| seen.insert(url.clone()))
        .collect()
}

fn block_links(block: &MarkdownBlock) -> Vec<String> {
    match block {
        MarkdownBlock::Heading { content, .. } | MarkdownBlock::Paragraph(content) => {
            inline_links(content)
        }
        MarkdownBlock::BulletList(items) => items.iter().flat_map(list_item_links).collect(),
        MarkdownBlock::OrderedList { items, .. } => {
            items.iter().flat_map(list_item_links).collect()
        }
        MarkdownBlock::Quote(blocks) => blocks.iter().flat_map(block_links).collect(),
        MarkdownBlock::Table(table) => {
            let mut urls = Vec::new();
            for header in &table.headers {
                urls.extend(inline_links(header));
            }
            for row in &table.rows {
                for cell in row {
                    urls.extend(inline_links(cell));
                }
            }
            urls
        }
        MarkdownBlock::CodeBlock { .. } | MarkdownBlock::ThematicBreak => Vec::new(),
    }
}

fn list_item_links(item: &ListItem) -> Vec<String> {
    item.blocks.iter().flat_map(block_links).collect()
}

fn inline_links(spans: &[InlineSpan]) -> Vec<String> {
    spans
        .iter()
        .filter_map(|span| match span {
            InlineSpan::Link { destination, .. } => Some(destination.clone()),
            _ => None,
        })
        .collect()
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

pub fn preprocess_markdown(markdown: &str, max_width: usize) -> String {
    parse_markdown_blocks(markdown)
        .into_iter()
        .map(|block| block_to_plain_text(block, max_width))
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
            Event::Html(html) => {
                // Skip HTML comments; render other block-level HTML as paragraph
                let html_str = html.to_string();
                let trimmed = html_str.trim();
                if !trimmed.starts_with("<!--") && !trimmed.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(vec![InlineSpan::Text(html_str)]));
                }
                *index += 1;
            }
            Event::DisplayMath(math) => {
                blocks.push(MarkdownBlock::CodeBlock {
                    language: Some("math".into()),
                    code: math.to_string(),
                });
                *index += 1;
            }
            e if is_loose_inline_event(e) => {
                blocks.push(MarkdownBlock::Paragraph(parse_loose_inlines(events, index)));
            }
            _ => {
                *index += 1;
            }
        }
    }

    blocks
}

fn parse_list_items<'a>(events: &[Event<'a>], index: &mut usize, ordered: bool) -> Vec<ListItem> {
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
            Event::Html(html) => {
                let html_str = html.to_string();
                let trimmed = html_str.trim();
                if !trimmed.starts_with("<!--") && !trimmed.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(vec![InlineSpan::Text(html_str)]));
                }
                *index += 1;
            }
            Event::DisplayMath(math) => {
                blocks.push(MarkdownBlock::CodeBlock {
                    language: Some("math".into()),
                    code: math.to_string(),
                });
                *index += 1;
            }
            e if is_loose_inline_event(e) => {
                blocks.push(MarkdownBlock::Paragraph(parse_loose_inlines(events, index)));
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
            Event::InlineHtml(html) => {
                let html_str = html.to_string();
                if !html_str.trim().starts_with("<!--") {
                    push_text(&mut spans, html_str);
                }
                *index += 1;
            }
            Event::FootnoteReference(label) => {
                push_text(&mut spans, format!("[^{}]", label));
                *index += 1;
            }
            Event::InlineMath(math) => {
                spans.push(InlineSpan::Code(format!("${}$", math)));
                *index += 1;
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
            Event::InlineHtml(html) => {
                if !html.trim().starts_with("<!--") {
                    text.push_str(html);
                }
                *index += 1;
            }
            Event::FootnoteReference(label) => {
                text.push_str(&format!("[^{}]", label));
                *index += 1;
            }
            Event::InlineMath(math) => {
                text.push_str(&format!("${}$", math));
                *index += 1;
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

/// True when pulldown-cmark emits an inline event outside of a Paragraph container.
/// These need to be collected into a synthetic Paragraph.
fn is_loose_inline_event(event: &Event<'_>) -> bool {
    matches!(
        event,
        Event::Text(_)
            | Event::Code(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::TaskListMarker(_)
            | Event::Start(Tag::Emphasis)
            | Event::Start(Tag::Strong)
            | Event::Start(Tag::Strikethrough)
            | Event::Start(Tag::Link { .. })
            | Event::Start(Tag::Image { .. })
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::InlineMath(_)
    )
}

/// Collect consecutive inline events into spans, stopping at any block-level event.
/// Used when pulldown-cmark emits bare inline events inside a list item or block context.
fn parse_loose_inlines<'a>(events: &[Event<'a>], index: &mut usize) -> Vec<InlineSpan> {
    let mut spans = Vec::new();

    while *index < events.len() {
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
            Event::InlineHtml(html) => {
                let html_str = html.to_string();
                if !html_str.trim().starts_with("<!--") {
                    push_text(&mut spans, html_str);
                }
                *index += 1;
            }
            Event::FootnoteReference(label) => {
                push_text(&mut spans, format!("[^{}]", label));
                *index += 1;
            }
            Event::InlineMath(math) => {
                spans.push(InlineSpan::Code(format!("${}$", math)));
                *index += 1;
            }
            _ => break,
        }
    }

    spans
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

pub fn calculate_column_widths(table: &TableBlock) -> Vec<usize> {
    use unicode_width::UnicodeWidthStr;
    let col_count = table.headers.len();
    let mut widths = vec![0usize; col_count];
    for (i, header) in table.headers.iter().enumerate() {
        widths[i] = UnicodeWidthStr::width(inline_text(header).as_str());
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                let w = UnicodeWidthStr::width(inline_text(cell).as_str());
                widths[i] = widths[i].max(w);
            }
        }
    }
    widths
}

fn block_to_plain_text(block: MarkdownBlock, max_width: usize) -> String {
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
            .map(|b| block_to_plain_text(b, max_width))
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::CodeBlock { language, code } => match language {
            Some(language) => format!("```{language}\n{code}```"),
            None => format!("```\n{code}```"),
        },
        MarkdownBlock::Table(table) => table_to_plain_text(&table, max_width),
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
        .map(|b| block_to_plain_text(b, 0))
        .collect::<Vec<_>>()
        .join(" ")
}

fn pad_to_width(text: &str, width: usize) -> String {
    use unicode_width::UnicodeWidthStr;
    let display_w = UnicodeWidthStr::width(text);
    if display_w >= width {
        text.to_string()
    } else {
        let mut s = text.to_string();
        s.push_str(&" ".repeat(width - display_w));
        s
    }
}

fn table_grid_width(col_widths: &[usize]) -> usize {
    col_widths.iter().sum::<usize>() + col_widths.len() * 3 + 1
}

fn table_to_plain_text(table: &TableBlock, max_width: usize) -> String {
    let col_widths = calculate_column_widths(table);
    let grid_needs = table_grid_width(&col_widths);
    if max_width > 0 && grid_needs <= max_width {
        table_to_grid_text(table, &col_widths)
    } else {
        table_to_collapsed_text(table)
    }
}

fn table_to_grid_text(table: &TableBlock, col_widths: &[usize]) -> String {
    let mut lines = Vec::new();

    let top = format!(
        "┌{}┐",
        col_widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┬")
    );
    lines.push(top);

    let header_cells: Vec<String> = table
        .headers
        .iter()
        .enumerate()
        .map(|(i, h)| pad_to_width(&inline_text(h), col_widths[i]))
        .collect();
    lines.push(format!("│ {} │", header_cells.join(" │ ")));

    let sep = format!(
        "├{}┤",
        col_widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┼")
    );
    lines.push(sep);

    for row in &table.rows {
        let cells: Vec<String> = col_widths
            .iter()
            .enumerate()
            .map(|(i, _)| {
                row.get(i)
                    .map(|cell| pad_to_width(&inline_text(cell), col_widths[i]))
                    .unwrap_or_else(|| " ".repeat(col_widths[i]))
            })
            .collect();
        lines.push(format!("│ {} │", cells.join(" │ ")));
    }

    let bottom = format!(
        "└{}┘",
        col_widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┴")
    );
    lines.push(bottom);

    lines.join("\n")
}

fn table_to_collapsed_text(table: &TableBlock) -> String {
    let mut rendered = vec![String::from("> [table collapsed for terminal width]")];
    let headers: Vec<String> = table.headers.iter().map(|cell| inline_text(cell)).collect();

    for (row_idx, row) in table.rows.iter().enumerate() {
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

        let rendered = preprocess_markdown(markdown, 10);

        assert!(rendered.contains("docs (https://docs.rs)"));
        assert!(rendered.contains("- [x] shipped"));
        assert!(rendered.contains("[table collapsed for terminal width]"));
        assert!(rendered.contains("- Name: Alice"));
        assert!(rendered.contains("- Role: Engineer"));
    }

    #[test]
    fn preprocess_markdown_renders_table_grid_when_fits() {
        let markdown = "\
| Name | Status |\n\
| --- | --- |\n\
| Parser | Active |";

        let rendered = preprocess_markdown(markdown, 80);

        assert!(rendered.contains("┌"));
        assert!(rendered.contains("│ Name"));
        assert!(rendered.contains("│ Parser"));
        assert!(!rendered.contains("[table collapsed"));
    }

    #[test]
    fn calculate_column_widths_handles_cjk() {
        let table = TableBlock {
            headers: vec![
                vec![InlineSpan::Text(String::from("模糊表达"))],
                vec![InlineSpan::Text(String::from("显化表达"))],
            ],
            rows: vec![vec![
                vec![InlineSpan::Text(String::from("弄好看一点"))],
                vec![InlineSpan::Text(String::from("abc"))],
            ]],
        };
        let widths = super::calculate_column_widths(&table);
        // "模糊表达" = 8 display width, "弄好看一点" = 10 display width
        assert_eq!(widths[0], 10);
        // "显化表达" = 8 display width, "abc" = 3 display width
        assert_eq!(widths[1], 8);
    }

    #[test]
    fn parse_nested_bullet_list_with_content_after() {
        let markdown = "- item 1\n  - sub a\n  - sub b\n- item 2\n\nafter list";
        let blocks = parse_markdown_blocks(markdown);

        assert_eq!(
            blocks,
            vec![
                MarkdownBlock::BulletList(vec![
                    ListItem {
                        checked: None,
                        blocks: vec![
                            MarkdownBlock::Paragraph(vec![InlineSpan::Text("item 1".into())]),
                            MarkdownBlock::BulletList(vec![
                                ListItem {
                                    checked: None,
                                    blocks: vec![MarkdownBlock::Paragraph(vec![InlineSpan::Text(
                                        "sub a".into()
                                    )])],
                                },
                                ListItem {
                                    checked: None,
                                    blocks: vec![MarkdownBlock::Paragraph(vec![InlineSpan::Text(
                                        "sub b".into()
                                    )])],
                                },
                            ]),
                        ],
                    },
                    ListItem {
                        checked: None,
                        blocks: vec![MarkdownBlock::Paragraph(vec![InlineSpan::Text(
                            "item 2".into()
                        )])],
                    },
                ]),
                MarkdownBlock::Paragraph(vec![InlineSpan::Text("after list".into())]),
            ],
            "content after nested bullet list must be parsed"
        );
    }

    #[test]
    fn parse_nested_bullet_with_content_before_and_after() {
        let markdown = "intro paragraph\n\n- item 1\n  - sub a\n  - sub b\n- item 2\n\nafter list";
        let blocks = parse_markdown_blocks(markdown);

        assert_eq!(
            blocks.len(),
            3,
            "should have 3 top-level blocks: paragraph, bullet list, paragraph. Got: {blocks:?}"
        );
        assert_eq!(
            blocks[2],
            MarkdownBlock::Paragraph(vec![InlineSpan::Text("after list".into())]),
            "content after nested bullet list was lost. Got: {:?}",
            blocks[2]
        );
    }

    #[test]
    fn parse_mixed_ordered_unordered_nested_lists() {
        let markdown =
            "- bullet\n  1. ordered sub 1\n  2. ordered sub 2\n- bullet 2\n\nfinal paragraph";
        let blocks = parse_markdown_blocks(markdown);

        assert_eq!(
            blocks.len(),
            2,
            "should have 2 blocks: mixed nested list and paragraph"
        );
        assert_eq!(
            blocks[1],
            MarkdownBlock::Paragraph(vec![InlineSpan::Text("final paragraph".into())]),
            "paragraph after mixed nested list was lost"
        );
    }

    #[test]
    fn parse_markdown_with_html_comment_inside_list() {
        let markdown = "- item 1\n  <!-- comment -->\n  - sub a\n- item 2\n\nafter";
        let blocks = parse_markdown_blocks(markdown);
        // HTML comments should be parsed (they become InlineHtml events which are handled)
        assert!(
            blocks.len() >= 2,
            "expected at least 2 blocks, got {blocks:?}"
        );
    }

    #[test]
    fn parse_markdown_with_text_after_list_no_blank_line() {
        // Some markdown parsers require blank line before new block after list,
        // but pulldown-cmark is more lenient
        let markdown = "- item 1\n- item 2\ntext without blank line";
        let blocks = parse_markdown_blocks(markdown);
        // pulldown-cmark: the "text without blank line" is a continuation of item 2
        // This is expected behavior - just documenting it
        assert!(!blocks.is_empty(), "should parse something");
    }

    #[test]
    fn parse_triple_nested_bullet_list() {
        let markdown = "intro\n\n- item 1\n  - sub a\n    - sub sub x\n    - sub sub y\n  - sub b\n- item 2\n\nafter all";
        let blocks = parse_markdown_blocks(markdown);

        // Should have: paragraph, bullet list, paragraph
        assert_eq!(
            blocks.len(),
            3,
            "expected 3 top-level blocks (paragraph, list, paragraph). Got: {blocks:#?}"
        );
        assert!(
            matches!(blocks[2], MarkdownBlock::Paragraph(_)),
            "third block should be paragraph 'after all'. Got: {:?}",
            blocks[2]
        );
    }

    #[test]
    fn parse_list_with_empty_items() {
        // List items with no content between them
        let markdown = "- \n- item\n- \n\nafter";
        let blocks = parse_markdown_blocks(markdown);
        assert!(
            blocks.len() >= 2,
            "expected list + paragraph after. Got: {blocks:#?}"
        );
    }

    #[test]
    fn parse_list_item_followed_by_thematic_break() {
        let markdown = "- item 1\n  - sub a\n\n---\n\nafter hr";
        let blocks = parse_markdown_blocks(markdown);
        // Should have: list, thematic break, paragraph
        assert_eq!(
            blocks.len(),
            3,
            "expected list, hr, paragraph. Got: {blocks:#?}"
        );
        assert!(
            matches!(blocks[1], MarkdownBlock::ThematicBreak),
            "expected thematic break as second block. Got: {:?}",
            blocks[1]
        );
        assert!(
            matches!(blocks[2], MarkdownBlock::Paragraph(_)),
            "expected paragraph 'after hr' as third block. Got: {:?}",
            blocks[2]
        );
    }

    #[test]
    fn html_events_are_not_silently_consumed() {
        // Block-level HTML in markdown
        let markdown = "before\n\n<div>html block</div>\n\nafter";
        let blocks = parse_markdown_blocks(markdown);
        // If Event::Html is consumed by catch-all, "html block" text disappears
        let rendered = markdown_to_debug_string(&blocks);
        assert!(
            rendered.contains("html block"),
            "HTML block content was consumed by catch-all! Blocks: {blocks:#?}"
        );
        assert!(
            rendered.contains("after"),
            "content after HTML block was lost! Rendered: {rendered}"
        );
    }

    #[test]
    fn inline_html_not_silently_consumed() {
        let markdown = "prefix <span>inline html</span> suffix";
        let blocks = parse_markdown_blocks(markdown);
        let rendered = markdown_to_debug_string(&blocks);
        assert!(
            rendered.contains("inline html") || rendered.contains("<span>"),
            "inline HTML was consumed! Blocks: {blocks:#?}"
        );
    }

    #[test]
    fn html_comment_not_breaking_parse() {
        // HTML comments should not break the parsing of surrounding content
        let markdown = "<!-- @Author: XPectuer -->\n# FAQ\n\n- item 1\n  - sub a\n\nafter";
        let blocks = parse_markdown_blocks(markdown);
        let rendered = markdown_to_debug_string(&blocks);
        assert!(
            rendered.contains("FAQ"),
            "content after HTML comment was lost! Rendered: {rendered}"
        );
        assert!(
            rendered.contains("after"),
            "content after list following HTML comment was lost! Rendered: {rendered}"
        );
    }

    fn markdown_to_debug_string(blocks: &[MarkdownBlock]) -> String {
        blocks
            .iter()
            .map(|b| format!("{b:?}"))
            .collect::<Vec<_>>()
            .join(" | ")
    }

    #[test]
    fn parse_list_item_with_inline_code_not_in_paragraph() {
        // pulldown-cmark emits bare Code/Text events inside list items
        // without wrapping them in Start(Paragraph)/End(Paragraph) when
        // inline formatting (like backtick code) is present.
        let markdown = "- 优化 `str_replace` → 返回值变了\n";
        let blocks = parse_markdown_blocks(markdown);
        assert_eq!(
            blocks,
            vec![MarkdownBlock::BulletList(vec![ListItem {
                checked: None,
                blocks: vec![MarkdownBlock::Paragraph(vec![
                    InlineSpan::Text("优化 ".into()),
                    InlineSpan::Code("str_replace".into()),
                    InlineSpan::Text(" → 返回值变了".into()),
                ])],
            }])],
            "inline code inside list item must not be lost. Got: {blocks:#?}"
        );
    }

    #[test]
    fn parse_list_item_with_strong_emphasis_outside_paragraph() {
        let markdown = "- `code` 自己的测试**通过了**\n";
        let blocks = parse_markdown_blocks(markdown);
        assert_eq!(
            blocks,
            vec![MarkdownBlock::BulletList(vec![ListItem {
                checked: None,
                blocks: vec![MarkdownBlock::Paragraph(vec![
                    InlineSpan::Code("code".into()),
                    InlineSpan::Text(" 自己的测试".into()),
                    InlineSpan::Strong("通过了".into()),
                ])],
            }])],
            "inline code and strong inside list item must be preserved. Got: {blocks:#?}"
        );
    }

    #[test]
    fn parse_full_slide_with_inline_code_and_blockquote_after_image() {
        let markdown = "\
# 测试 = 模具\n\
\n\
**uigen 进化史 v3**：核心模块加测试，precommit hook 挂测试。\n\
\n\
## 痛点：改 A 修 B 炸 C\n\
\n\
uigen 实例：\n\
- 优化 `str_replace` → 返回值从 `{ content }` 变成 `{ content, changed }`\n\
- `str-replace.ts` 自己的测试**通过了**\n\
- 但 `generation.tsx` 解析返回值报错 → 全部组件白屏\n\
\n\
→ 典型**回归**。AI 精确执行任务，但不会主动检查\"我改的会影响其他模块吗\"。\n\
\n\
> AI 改代码快 10 倍，埋 bug 也快 10 倍。测试比传统开发更重要。\n";
        let blocks = parse_markdown_blocks(markdown);

        // The last two blocks should be: paragraph with "→ 典型..." and blockquote
        assert!(
            blocks.len() >= 7,
            "expected at least 7 blocks, got {blocks:#?}"
        );

        // Find the blockquote
        let has_blockquote = blocks.iter().any(|b| matches!(b, MarkdownBlock::Quote(_)));
        assert!(
            has_blockquote,
            "blockquote 'AI 改代码快 10 倍...' was lost! Blocks: {blocks:#?}"
        );

        // Find the paragraph with "我改的会影响其他模块吗"
        let rendered = markdown_to_debug_string(&blocks);
        assert!(
            rendered.contains("我改的会影响其他模块吗"),
            "paragraph text '我改的会影响其他模块吗' was lost! Rendered: {rendered}"
        );
        assert!(
            rendered.contains("AI 改代码快 10 倍"),
            "blockquote text 'AI 改代码快 10 倍' was lost! Rendered: {rendered}"
        );
        assert!(
            rendered.contains("str_replace"),
            "inline code 'str_replace' was lost! Rendered: {rendered}"
        );
    }

    #[test]
    fn parse_list_item_with_link_and_strikethrough() {
        let markdown = "- see [docs](https://example.com) and ~~old~~\n";
        let blocks = parse_markdown_blocks(markdown);
        assert_eq!(
            blocks,
            vec![MarkdownBlock::BulletList(vec![ListItem {
                checked: None,
                blocks: vec![MarkdownBlock::Paragraph(vec![
                    InlineSpan::Text("see ".into()),
                    InlineSpan::Link {
                        label: "docs".into(),
                        destination: "https://example.com".into(),
                    },
                    InlineSpan::Text(" and ".into()),
                    InlineSpan::Strikethrough("old".into()),
                ])],
            }])],
            "link and strikethrough inside list item must be preserved. Got: {blocks:#?}"
        );
    }

    #[test]
    fn extract_headings_returns_plain_heading_text() {
        let markdown = "# Title\n\n## Subtitle with `code`";

        let headings = super::extract_headings(markdown);

        assert_eq!(
            headings,
            vec![String::from("Title"), String::from("Subtitle with code"),]
        );
    }
}
