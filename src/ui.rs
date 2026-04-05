use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    DefaultTerminal, Frame,
};
use ratatui_image::{Resize, StatefulImage};

use crate::{
    app::{App, Mode},
    image::{self, ImageRender},
    loader::Slide,
    markdown::{self, SlideBlock},
};

pub fn init_terminal() -> Result<DefaultTerminal> {
    Ok(ratatui::init())
}

pub fn restore_terminal() -> Result<()> {
    ratatui::restore();
    Ok(())
}

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.mode {
        Mode::Browse => render_browse(frame, app),
        Mode::Present => render_present(frame, app),
    }
}

fn render_browse(frame: &mut Frame, app: &mut App) {
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(1)])
        .split(frame.area());

    let nav = app
        .slides
        .iter()
        .enumerate()
        .map(|(idx, slide)| {
            if idx == app.selected {
                format!("> {}", slide.title)
            } else {
                format!("  {}", slide.title)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    frame.render_widget(
        Paragraph::new(nav).block(Block::default().title("Slides").borders(Borders::ALL)),
        areas[0],
    );

    let current = app.current_slide().clone();
    let preview = Block::default()
        .title(current.title.clone())
        .borders(Borders::ALL);
    let inner = preview.inner(areas[1]);
    frame.render_widget(preview, areas[1]);
    render_slide_blocks(frame, inner, app, &current);
}

fn render_present(frame: &mut Frame, app: &mut App) {
    let current = app.current_slide().clone();
    render_slide_blocks(frame, frame.area(), app, &current);
}

fn render_slide_blocks(frame: &mut Frame, area: Rect, app: &mut App, current: &Slide) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let base_dir = current
        .path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let mut cursor_y = i32::from(area.y) - i32::from(app.scroll);

    for block in markdown::parse_blocks(&current.raw_markdown) {
        let height = block_height(&block, area.width);
        if let Some((visible, text_scroll)) = clip_rect(area, cursor_y, height) {
            match block {
                SlideBlock::Markdown(blocks) => {
                    render_markdown_block(frame, visible, &blocks, text_scroll);
                }
                SlideBlock::Image { src, .. } => {
                    render_image_block(frame, visible, app, base_dir, &src);
                }
            }
        }

        cursor_y += i32::from(height) + 1;
        if cursor_y >= i32::from(area.y + area.height) {
            break;
        }
    }
}

fn render_markdown_block(
    frame: &mut Frame,
    area: Rect,
    blocks: &[markdown::MarkdownBlock],
    text_scroll: u16,
) {
    let rendered = render_markdown_text(blocks);
    let paragraph = Paragraph::new(rendered)
        .wrap(Wrap { trim: false })
        .scroll((text_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_markdown_text(blocks: &[markdown::MarkdownBlock]) -> Text<'static> {
    let mut lines = Vec::new();

    for (idx, block) in blocks.iter().enumerate() {
        push_block_lines(&mut lines, block, "");
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

fn push_block_lines(lines: &mut Vec<Line<'static>>, block: &markdown::MarkdownBlock, prefix: &str) {
    match block {
        markdown::MarkdownBlock::Heading { content, .. } => {
            let mut line = Line::from(prefixed_spans(prefix, render_inline_spans(content)));
            line.alignment = Some(Alignment::Center);
            line.style = heading_style();
            lines.push(line);
        }
        markdown::MarkdownBlock::Paragraph(content) => {
            lines.push(Line::from(prefixed_spans(prefix, render_inline_spans(content))));
        }
        markdown::MarkdownBlock::BulletList(items) => {
            for item in items {
                push_list_item_lines(lines, item, prefix, None);
            }
        }
        markdown::MarkdownBlock::OrderedList { start, items } => {
            for (idx, item) in items.iter().enumerate() {
                push_list_item_lines(lines, item, prefix, Some(format!("{}. ", start + idx)));
            }
        }
        markdown::MarkdownBlock::Quote(blocks) => {
            for (idx, block) in blocks.iter().enumerate() {
                push_block_lines(lines, block, &format!("{prefix}> "));
                if idx + 1 < blocks.len() && !line_is_blank(lines.last()) {
                    lines.push(Line::default());
                }
            }
        }
        markdown::MarkdownBlock::CodeBlock { language, code } => {
            if let Some(language) = language {
                lines.push(Line::styled(
                    format!("{prefix}[code:{language}]"),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
            lines.push(Line::from(format!("{prefix}> [table collapsed for terminal width]")));
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
) {
    let list_prefix = ordered_prefix.unwrap_or_else(|| match item.checked {
        Some(true) => String::from("- [x] "),
        Some(false) => String::from("- [ ] "),
        None => String::from("- "),
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
        push_block_lines(lines, block, &item_prefix);
    }
}

fn render_inline_spans(spans: &[markdown::InlineSpan]) -> Vec<Span<'static>> {
    spans.iter().flat_map(render_inline_span).collect()
}

fn render_inline_span(span: &markdown::InlineSpan) -> Vec<Span<'static>> {
    match span {
        markdown::InlineSpan::Text(text) => vec![Span::raw(text.clone())],
        markdown::InlineSpan::Strong(text) => {
            vec![Span::styled(text.clone(), Style::default().add_modifier(Modifier::BOLD))]
        }
        markdown::InlineSpan::Emphasis(text) => {
            vec![Span::styled(text.clone(), Style::default().add_modifier(Modifier::ITALIC))]
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
    app: &mut App,
    base_dir: &std::path::Path,
    src: &str,
) {
    match image::prepare_image(base_dir, src) {
        Ok(ImageRender::TerminalImage { path }) => match app.image_state_for(&path) {
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

fn block_height(block: &SlideBlock, width: u16) -> u16 {
    match block {
        SlideBlock::Markdown(blocks) => {
            let rendered = render_markdown_text(blocks);
            estimate_text_height(&rendered, width.max(1))
        }
        SlideBlock::Image { .. } => 12,
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
            SlideBlock::Markdown(blocks) => text_to_string(render_markdown_text(&blocks)),
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
    use super::render;
    use crate::{app::App, loader::Slide};
    use ratatui::{backend::TestBackend, Terminal};
    use ratatui_image::picker::Picker;
    use std::{
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

    #[test]
    fn render_outputs_text_and_missing_image_fallback_in_browse_mode() {
        let dir = TempDir::new("ui-render");
        let slides = vec![Slide {
            path: dir.path().join("01.md"),
            title: String::from("01"),
            raw_markdown: String::from("# Title\n\nBody\n\n![diagram](missing.png)"),
        }];
        let mut app = App {
            slides,
            selected: 0,
            mode: crate::app::Mode::Browse,
            scroll: 0,
            should_quit: false,
            image_picker: None,
            image_states: std::collections::HashMap::new(),
        };
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| render(frame, &mut app)).unwrap();
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

        let slides = vec![Slide {
            path: dir.path().join("01.md"),
            title: String::from("01"),
            raw_markdown: String::from("# Title\n\n![diagram](photo.png)"),
        }];
        let mut app = App {
            slides,
            selected: 0,
            mode: crate::app::Mode::Browse,
            scroll: 0,
            should_quit: false,
            image_picker: Some(Picker::from_fontsize((8, 16))),
            image_states: std::collections::HashMap::new(),
        };
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| render(frame, &mut app)).unwrap();
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
