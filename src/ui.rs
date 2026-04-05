use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
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
        let height = block_height(&block);
        if let Some((visible, text_scroll)) = clip_rect(area, cursor_y, height) {
            match block {
                SlideBlock::Text(text) => {
                    frame.render_widget(Paragraph::new(text).scroll((text_scroll, 0)), visible);
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

fn block_height(block: &SlideBlock) -> u16 {
    match block {
        SlideBlock::Text(text) => text.lines().count().max(1) as u16,
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
            SlideBlock::Text(text) => text,
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
