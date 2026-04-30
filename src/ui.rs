use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    DefaultTerminal, Frame,
};
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};
use std::path::Path;

use crate::{
    image::{self, ImageRender},
    loader::{Slide, SlideNode, SlideRef, VisibleItem, VisibleItemKind},
    markdown::{self, SlideBlock},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Browse,
    Present,
}

pub struct RenderModel<'a> {
    pub nodes: &'a [SlideNode],
    pub visible: &'a [VisibleItem],
    pub flat_refs: &'a [SlideRef],
    pub selected: usize,
    pub present_index: usize,
    pub mode: RenderMode,
    pub scroll: u16,
}

pub trait ImageStateStore {
    fn image_state_for(&mut self, path: &Path) -> Result<&mut StatefulProtocol>;
}

pub fn init_terminal() -> Result<DefaultTerminal> {
    Ok(ratatui::init())
}

pub fn restore_terminal() -> Result<()> {
    ratatui::restore();
    Ok(())
}

pub fn render(frame: &mut Frame, model: &RenderModel<'_>, image_states: &mut dyn ImageStateStore) {
    match model.mode {
        RenderMode::Browse => render_browse(frame, model, image_states),
        RenderMode::Present => render_present(frame, model, image_states),
    }
}

pub fn render_reload_indicator(frame: &mut Frame) {
    let text = " Reloaded ";
    let width = u16::try_from(text.len()).unwrap_or(10);
    let area = Rect {
        x: frame.area().width.saturating_sub(width + 2),
        y: frame.area().height.saturating_sub(2),
        width: width + 2,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(text).style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        area,
    );
}

fn render_browse(
    frame: &mut Frame,
    model: &RenderModel<'_>,
    image_states: &mut dyn ImageStateStore,
) {
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Min(1)])
        .split(frame.area());

    let nav_lines: Vec<Line> = model
        .visible
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_selected = idx == model.selected;
            let prefix = if is_selected { "> " } else { "  " };
            let indent = if item.depth == 1 { "  " } else { "" };

            match &item.kind {
                VisibleItemKind::RootLeaf { .. } | VisibleItemKind::GroupChild { .. } => {
                    let title = item
                        .slide_ref
                        .as_ref()
                        .map(|r| SlideNode::resolve_slide(model.nodes, r).title.as_str())
                        .unwrap_or("?");
                    let style = if item.depth == 1 {
                        Style::default().fg(Color::Gray)
                    } else {
                        Style::default()
                    };
                    Line::styled(format!("{prefix}{indent}{title}"), style)
                }
                VisibleItemKind::Group {
                    name,
                    expanded,
                    child_count,
                } => {
                    let arrow = if *expanded { "▼" } else { "▶" };
                    let style = Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD);
                    Line::styled(
                        format!("{prefix}{indent}{arrow} {name} ({child_count})"),
                        style,
                    )
                }
            }
        })
        .collect();

    frame.render_widget(
        Paragraph::new(nav_lines).block(Block::default().title("Slides").borders(Borders::ALL)),
        areas[0],
    );

    let current_slide = model
        .visible
        .get(model.selected)
        .and_then(|item| item.slide_ref.as_ref())
        .map(|r| SlideNode::resolve_slide(model.nodes, r).clone());

    match current_slide {
        Some(current) => {
            let preview = Block::default()
                .title(current.title.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White));
            let inner = preview.inner(areas[1]);
            frame.render_widget(preview, areas[1]);
            render_slide_blocks(frame, inner, areas[1], model, image_states, &current);
        }
        None => {
            // Group selected — show summary
            let group_info = model.visible.get(model.selected).and_then(|item| {
                if let VisibleItemKind::Group {
                    name,
                    child_count,
                    expanded,
                } = &item.kind
                {
                    Some((name.clone(), *child_count, *expanded))
                } else {
                    None
                }
            });
            let text = match group_info {
                Some((name, count, expanded)) => {
                    let hint = if expanded {
                        "Press Enter to collapse"
                    } else {
                        "Press Enter to expand"
                    };
                    format!("  {name}\n\n  {count} slides in this group\n\n  {hint}")
                }
                None => String::from("  No slide selected"),
            };
            let preview = Block::default()
                .title("Group")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            let inner = preview.inner(areas[1]);
            frame.render_widget(preview, areas[1]);
            frame.render_widget(Paragraph::new(text), inner);
        }
    }
}

fn render_present(
    frame: &mut Frame,
    model: &RenderModel<'_>,
    image_states: &mut dyn ImageStateStore,
) {
    let current = model
        .flat_refs
        .get(model.present_index)
        .map(|r| SlideNode::resolve_slide(model.nodes, r).clone());

    let Some(current) = current else {
        return;
    };

    let content_width = frame.area().width.saturating_sub(2);
    let content_height = slide_content_height(&current.raw_markdown, content_width);
    let max_inner = frame.area().height.saturating_sub(2);

    let border_height = if content_height > max_inner {
        frame.area().height
    } else {
        content_height.saturating_add(3).min(frame.area().height)
    };

    let area = Rect {
        height: border_height,
        ..frame.area()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    render_slide_blocks(frame, inner, area, model, image_states, &current);
}

fn slide_content_height(raw_markdown: &str, width: u16) -> u16 {
    text_content_height(raw_markdown, width)
}

fn text_content_height(raw_markdown: &str, width: u16) -> u16 {
    let mut total: u16 = 0;
    for block in &markdown::parse_blocks(raw_markdown) {
        if let SlideBlock::Markdown(_) = block {
            total = total.saturating_add(block_height(block, width, 0));
            total = total.saturating_add(1);
        }
    }
    total.max(1)
}

fn text_content_height_from_blocks(blocks: &[SlideBlock], width: u16) -> u16 {
    let mut total: u16 = 0;
    for block in blocks {
        if let SlideBlock::Markdown(_) = block {
            total = total.saturating_add(block_height(block, width, 0));
            total = total.saturating_add(1);
        }
    }
    total.max(1)
}

fn render_slide_blocks(
    frame: &mut Frame,
    inner: Rect,
    border_area: Rect,
    model: &RenderModel<'_>,
    image_states: &mut dyn ImageStateStore,
    current: &Slide,
) {
    let base_dir = current
        .path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let blocks = markdown::parse_blocks(&current.raw_markdown);

    // Render text blocks inside the bordered area
    render_text_blocks(frame, inner, model, &blocks);

    // Render image blocks below the bordered area, or fallback inside if no space below
    render_image_blocks_below(
        frame,
        border_area,
        inner,
        image_states,
        base_dir,
        model,
        &blocks,
    );
}

fn render_text_blocks(
    frame: &mut Frame,
    area: Rect,
    model: &RenderModel<'_>,
    blocks: &[SlideBlock],
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let mut cursor_y = i32::from(area.y) - i32::from(model.scroll);
    for block in blocks {
        if let SlideBlock::Markdown(ref md_blocks) = block {
            let height = block_height(block, area.width, 0);
            if let Some((visible, text_scroll)) = clip_rect(area, cursor_y, height) {
                render_markdown_block(frame, visible, md_blocks, text_scroll);
            }
            cursor_y += i32::from(height) + 1;
            if cursor_y >= i32::from(area.y + area.height) {
                return;
            }
        }
    }
}

fn render_image_blocks_below(
    frame: &mut Frame,
    border_area: Rect,
    inner: Rect,
    image_states: &mut dyn ImageStateStore,
    base_dir: &Path,
    model: &RenderModel<'_>,
    blocks: &[SlideBlock],
) {
    // Images start below the border: y = border_area.y + border_area.height
    let start_y = border_area.y + border_area.height;
    let frame_bottom = frame.area().y + frame.area().height;
    let has_space_below = start_y < frame_bottom;

    let image_area = if has_space_below {
        Rect {
            y: start_y,
            height: frame_bottom.saturating_sub(start_y),
            x: border_area.x + 1,
            width: border_area.width.saturating_sub(2),
        }
    } else {
        // Fallback: render images inside the border, below text
        let text_h = text_content_height_from_blocks(blocks, inner.width);
        Rect {
            y: inner.y + text_h,
            height: inner.height.saturating_sub(text_h),
            x: inner.x,
            width: inner.width,
        }
    };

    if image_area.width == 0 || image_area.height == 0 {
        return;
    }

    let mut cursor_y = i32::from(image_area.y) - i32::from(model.scroll);
    for block in blocks {
        if let SlideBlock::Image { ref src, .. } = block {
            let height = block_height(block, image_area.width, image_area.height);
            if let Some((visible, _)) = clip_rect(image_area, cursor_y, height) {
                render_image_block(frame, visible, image_states, base_dir, src);
            }
            cursor_y += i32::from(height) + 1;
            if cursor_y >= i32::from(image_area.y + image_area.height) {
                return;
            }
        }
    }
}

fn render_markdown_block(
    frame: &mut Frame,
    area: Rect,
    blocks: &[markdown::MarkdownBlock],
    text_scroll: u16,
) {
    let rendered = render_markdown_text(blocks, area.width);
    let paragraph = Paragraph::new(rendered)
        .wrap(Wrap { trim: false })
        .scroll((text_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_markdown_text(blocks: &[markdown::MarkdownBlock], width: u16) -> Text<'static> {
    let mut lines = Vec::new();

    for (idx, block) in blocks.iter().enumerate() {
        push_block_lines(&mut lines, block, "", width);
        if idx + 1 < blocks.len() && !line_is_blank(lines.last()) {
            lines.push(Line::default());
        }
    }

    while line_is_blank(lines.last()) {
        lines.pop();
    }

    if lines.is_empty() {
        lines.push(Line::default());
    }

    Text::from(lines)
}

fn push_block_lines(
    lines: &mut Vec<Line<'static>>,
    block: &markdown::MarkdownBlock,
    prefix: &str,
    width: u16,
) {
    match block {
        markdown::MarkdownBlock::Heading { content, .. } => {
            let mut line = Line::from(prefixed_spans(prefix, render_inline_spans(content)));
            line.alignment = Some(Alignment::Center);
            line.style = heading_style();
            lines.push(line);
        }
        markdown::MarkdownBlock::Paragraph(content) => {
            lines.push(Line::from(prefixed_spans(
                prefix,
                render_inline_spans(content),
            )));
        }
        markdown::MarkdownBlock::BulletList(items) => {
            for item in items {
                push_list_item_lines(lines, item, prefix, None, width);
            }
        }
        markdown::MarkdownBlock::OrderedList { start, items } => {
            for (idx, item) in items.iter().enumerate() {
                push_list_item_lines(
                    lines,
                    item,
                    prefix,
                    Some(format!("{}. ", start + idx)),
                    width,
                );
            }
        }
        markdown::MarkdownBlock::Quote(blocks) => {
            let quote_style = Style::default().bg(Color::DarkGray);
            let bar_style = Style::default().fg(Color::Cyan);
            let start = lines.len();
            for (idx, block) in blocks.iter().enumerate() {
                push_block_lines(lines, block, &format!("{prefix}  "), width);
                if idx + 1 < blocks.len() && !line_is_blank(lines.last()) {
                    lines.push(Line::default());
                }
            }
            // Apply quote styling: left bar + background
            for line in lines.iter_mut().skip(start) {
                let bar = Span::styled("│ ", bar_style);
                let mut spans: Vec<Span<'static>> = vec![bar];
                let old_spans = std::mem::take(line);
                let old_style = old_spans.style;
                spans.extend(old_spans.spans);
                let merged = old_style.patch(quote_style);
                *line = Line {
                    spans,
                    style: merged,
                    alignment: old_spans.alignment,
                };
            }
        }
        markdown::MarkdownBlock::CodeBlock { language, code } => {
            if let Some(language) = language {
                lines.push(Line::styled(
                    format!("{prefix}[code:{language}]"),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            for code_line in code.lines() {
                lines.push(Line::styled(
                    format!("{prefix}    {code_line}"),
                    Style::default().fg(Color::Green),
                ));
            }
        }
        markdown::MarkdownBlock::Table(table) => {
            let col_widths = markdown::calculate_column_widths(table);
            let prefix_len = prefix
                .chars()
                .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0))
                .sum::<usize>();
            let available = width as usize;
            let grid_needs = prefix_len + table_grid_width(&col_widths);
            if available > 0 && grid_needs <= available {
                lines.extend(render_table_grid(table, &col_widths, prefix));
            } else {
                lines.extend(render_table_collapsed(table, prefix));
            }
        }
        markdown::MarkdownBlock::ThematicBreak => {
            lines.push(Line::from(format!("{prefix}---")));
        }
    }
}

fn heading_style() -> Style {
    Style::default()
        .fg(Color::LightYellow)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

fn push_list_item_lines(
    lines: &mut Vec<Line<'static>>,
    item: &markdown::ListItem,
    prefix: &str,
    ordered_prefix: Option<String>,
    width: u16,
) {
    let list_prefix = ordered_prefix.unwrap_or_else(|| match item.checked {
        Some(true) => String::from("- [x] "),
        Some(false) => String::from("- [ ] "),
        None => String::from("• "),
    });

    if item.blocks.is_empty() {
        lines.push(Line::from(format!("{prefix}{list_prefix}")));
        return;
    }

    for (idx, block) in item.blocks.iter().enumerate() {
        let item_prefix = if idx == 0 {
            format!("{prefix}{list_prefix}")
        } else {
            format!("{prefix}{}", " ".repeat(list_prefix.chars().count()))
        };
        push_block_lines(lines, block, &item_prefix, width);
    }
}

fn render_inline_spans(spans: &[markdown::InlineSpan]) -> Vec<Span<'static>> {
    spans.iter().flat_map(render_inline_span).collect()
}

fn render_inline_span(span: &markdown::InlineSpan) -> Vec<Span<'static>> {
    match span {
        markdown::InlineSpan::Text(text) => vec![Span::raw(text.clone())],
        markdown::InlineSpan::Strong(text) => {
            vec![Span::styled(
                text.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )]
        }
        markdown::InlineSpan::Emphasis(text) => {
            vec![Span::styled(
                text.clone(),
                Style::default().add_modifier(Modifier::ITALIC),
            )]
        }
        markdown::InlineSpan::Strikethrough(text) => vec![Span::styled(
            text.clone(),
            Style::default().add_modifier(Modifier::CROSSED_OUT),
        )],
        markdown::InlineSpan::Code(text) => vec![Span::styled(
            text.clone(),
            Style::default().fg(Color::Green).bg(Color::DarkGray),
        )],
        markdown::InlineSpan::Link { label, destination } => vec![
            Span::styled(
                label.clone(),
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::raw(format!(" ({destination})")),
        ],
    }
}

fn prefixed_spans(prefix: &str, spans: Vec<Span<'static>>) -> Vec<Span<'static>> {
    let mut prefixed = Vec::with_capacity(spans.len() + 1);
    if !prefix.is_empty() {
        prefixed.push(Span::raw(prefix.to_string()));
    }
    prefixed.extend(spans);
    prefixed
}

fn inline_plain_text(spans: &[markdown::InlineSpan]) -> String {
    spans
        .iter()
        .map(|span| match span {
            markdown::InlineSpan::Text(text)
            | markdown::InlineSpan::Strong(text)
            | markdown::InlineSpan::Emphasis(text)
            | markdown::InlineSpan::Strikethrough(text)
            | markdown::InlineSpan::Code(text) => text.clone(),
            markdown::InlineSpan::Link { label, destination } => {
                format!("{label} ({destination})")
            }
        })
        .collect::<String>()
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

fn render_table_grid(
    table: &markdown::TableBlock,
    col_widths: &[usize],
    prefix: &str,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let top_border = format!(
        "{prefix}┌{}┐",
        col_widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┬")
    );
    lines.push(Line::from(top_border));

    // Header row
    let header_cells: Vec<String> = table
        .headers
        .iter()
        .enumerate()
        .map(|(i, h)| pad_to_width(&inline_plain_text(h), col_widths[i]))
        .collect();
    let header_line = format!("{prefix}│ {} │", header_cells.join(" │ "));
    lines.push(Line::styled(
        header_line,
        Style::default().add_modifier(Modifier::BOLD),
    ));

    // Header separator: ├───┼───┤
    let sep = format!(
        "{prefix}├{}┤",
        col_widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┼")
    );
    lines.push(Line::from(sep));

    // Data rows
    for row in &table.rows {
        let cells: Vec<String> = col_widths
            .iter()
            .enumerate()
            .map(|(i, _)| {
                row.get(i)
                    .map(|cell| pad_to_width(&inline_plain_text(cell), col_widths[i]))
                    .unwrap_or_else(|| " ".repeat(col_widths[i]))
            })
            .collect();
        let row_line = format!("{prefix}│ {} │", cells.join(" │ "));
        lines.push(Line::from(row_line));
    }

    // Bottom border: └───┴───┘
    let bottom_border = format!(
        "{prefix}└{}┘",
        col_widths
            .iter()
            .map(|w| "─".repeat(*w + 2))
            .collect::<Vec<_>>()
            .join("┴")
    );
    lines.push(Line::from(bottom_border));

    lines
}

fn render_table_collapsed(table: &markdown::TableBlock, prefix: &str) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(format!(
        "{prefix}> [table collapsed for terminal width]"
    ))];
    for (row_idx, row) in table.rows.iter().enumerate() {
        lines.push(Line::default());
        lines.push(Line::styled(
            format!("{prefix}**Row {}**", row_idx + 1),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        for (col_idx, header) in table.headers.iter().enumerate() {
            let value = row
                .get(col_idx)
                .map(|cell| inline_plain_text(cell))
                .unwrap_or_default();
            lines.push(Line::from(format!(
                "{prefix}- {}: {}",
                inline_plain_text(header),
                value
            )));
        }
    }
    lines
}

fn plain_line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>()
}

fn line_is_blank(line: Option<&Line<'_>>) -> bool {
    line.is_none_or(|line| plain_line_text(line).trim().is_empty())
}

fn estimate_text_height(text: &Text<'_>, width: u16) -> u16 {
    let width = width.max(1) as usize;
    text.lines
        .iter()
        .map(|line| {
            let plain = plain_line_text(line);
            let display_width = unicode_width::UnicodeWidthStr::width(plain.as_str());
            display_width.max(1).div_ceil(width) as u16
        })
        .sum::<u16>()
        .max(1)
}

fn render_image_block(
    frame: &mut Frame,
    area: Rect,
    image_states: &mut dyn ImageStateStore,
    base_dir: &std::path::Path,
    src: &str,
) {
    match image::prepare_image(base_dir, src) {
        Ok(ImageRender::TerminalImage { path }) => match image_states.image_state_for(&path) {
            Ok(state) => frame.render_stateful_widget(
                StatefulImage::default().resize(Resize::Fit(None)),
                area,
                state,
            ),
            Err(err) => frame.render_widget(Paragraph::new(format!("[image error] {err}")), area),
        },
        Ok(ImageRender::FallbackText { message }) => {
            frame.render_widget(Paragraph::new(message), area);
        }
        Err(err) => frame.render_widget(Paragraph::new(format!("[image error] {err}")), area),
    }
}

fn block_height(block: &SlideBlock, width: u16, image_max_height: u16) -> u16 {
    match block {
        SlideBlock::Markdown(blocks) => {
            let rendered = render_markdown_text(blocks, width);
            estimate_text_height(&rendered, width.max(1))
        }
        SlideBlock::Image { .. } => {
            if image_max_height == 0 {
                12
            } else {
                image_max_height.min(150)
            }
        }
    }
}

fn clip_rect(area: Rect, top: i32, height: u16) -> Option<(Rect, u16)> {
    let area_top = i32::from(area.y);
    let area_bottom = i32::from(area.y + area.height);
    let bottom = top + i32::from(height);
    if bottom <= area_top || top >= area_bottom {
        return None;
    }

    let visible_top = top.max(area_top);
    let visible_bottom = bottom.min(area_bottom);
    let text_scroll = if top < area_top {
        (area_top - top) as u16
    } else {
        0
    };

    Some((
        Rect::new(
            area.x,
            visible_top as u16,
            area.width,
            (visible_bottom - visible_top) as u16,
        ),
        text_scroll,
    ))
}

pub fn render_slide_content(base_dir: Option<&std::path::Path>, raw_markdown: &str) -> String {
    markdown::parse_blocks(raw_markdown)
        .into_iter()
        .map(|block| match block {
            SlideBlock::Markdown(blocks) => text_to_string(render_markdown_text(&blocks, 80)),
            SlideBlock::Image { src, .. } => {
                let base = base_dir.unwrap_or_else(|| std::path::Path::new("."));
                match image::prepare_image(base, &src) {
                    Ok(ImageRender::TerminalImage { path }) => {
                        format!("[image render] {}", path.display())
                    }
                    Ok(ImageRender::FallbackText { message }) => message,
                    Err(err) => format!("[image error] {err}"),
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn text_to_string(text: Text<'_>) -> String {
    text.lines
        .into_iter()
        .map(|line| plain_line_text(&line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{render, RenderMode, RenderModel};
    use crate::{
        app::{App, ImageContext},
        loader::{compute_flat_refs, compute_visible_items, Slide, SlideNode},
    };
    use ratatui::{backend::TestBackend, Terminal};
    use ratatui_image::picker::Picker;
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("slidet-{label}-{nanos}"));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn make_app_with_leaves(dir: &Path) -> App {
        let nodes = vec![SlideNode::Leaf(Slide {
            path: dir.join("01.md"),
            title: String::from("01"),
            raw_markdown: String::from("# Title\n\nBody\n\n![diagram](missing.png)"),
        })];
        let visible = compute_visible_items(&nodes);
        let flat_refs = compute_flat_refs(&nodes);
        App {
            nodes,
            visible,
            flat_refs,
            selected: 0,
            present_index: 0,
            mode: crate::app::Mode::Browse,
            scroll: 0,
            should_quit: false,
            image: ImageContext {
                image_picker: None,
                image_states: HashMap::new(),
            },
            slides_dir: dir.to_path_buf(),
            watcher: None,
            reload_indicator: None,
        }
    }

    #[test]
    fn render_outputs_text_and_missing_image_fallback_in_browse_mode() {
        let dir = TempDir::new("ui-render");
        let mut app = make_app_with_leaves(dir.path());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let model = RenderModel {
                    nodes: &app.nodes,
                    visible: &app.visible,
                    flat_refs: &app.flat_refs,
                    selected: app.selected,
                    present_index: app.present_index,
                    mode: RenderMode::Browse,
                    scroll: app.scroll,
                };
                render(frame, &model, &mut app.image)
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let screen = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(screen.contains("Title"));
        assert!(screen.contains("Body"));
        assert!(screen.contains("[missing image]"));
    }

    #[test]
    fn render_does_not_emit_string_placeholder_for_renderable_images() {
        let dir = TempDir::new("ui-image");
        let image_path = dir.path().join("photo.png");
        image::DynamicImage::new_rgba8(1, 1)
            .save(&image_path)
            .unwrap();
        let previous = std::env::var_os("KITTY_WINDOW_ID");
        std::env::set_var("KITTY_WINDOW_ID", "test-window");

        let nodes = vec![SlideNode::Leaf(Slide {
            path: dir.path().join("01.md"),
            title: String::from("01"),
            raw_markdown: String::from("# Title\n\n![diagram](photo.png)"),
        })];
        let visible = compute_visible_items(&nodes);
        let flat_refs = compute_flat_refs(&nodes);
        let mut app = App {
            nodes,
            visible,
            flat_refs,
            selected: 0,
            present_index: 0,
            mode: crate::app::Mode::Browse,
            scroll: 0,
            should_quit: false,
            image: ImageContext {
                image_picker: Some(Picker::from_fontsize((8, 16))),
                image_states: HashMap::new(),
            },
            slides_dir: dir.path().to_path_buf(),
            watcher: None,
            reload_indicator: None,
        };
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let model = RenderModel {
                    nodes: &app.nodes,
                    visible: &app.visible,
                    flat_refs: &app.flat_refs,
                    selected: app.selected,
                    present_index: app.present_index,
                    mode: RenderMode::Browse,
                    scroll: app.scroll,
                };
                render(frame, &model, &mut app.image)
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let screen = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        match previous {
            Some(value) => std::env::set_var("KITTY_WINDOW_ID", value),
            None => std::env::remove_var("KITTY_WINDOW_ID"),
        }

        assert!(screen.contains("Title"));
        assert!(!screen.contains("[image render]"));
    }
}
