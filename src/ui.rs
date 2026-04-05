use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};

use crate::{
    app::{App, Mode},
    image::{self, ImageRender},
    markdown::{self, SlideBlock},
};

pub fn init_terminal() -> Result<DefaultTerminal> {
    Ok(ratatui::init())
}

pub fn restore_terminal() -> Result<()> {
    ratatui::restore();
    Ok(())
}

pub fn render(frame: &mut Frame, app: &App) {
    match app.mode {
        Mode::Browse => render_browse(frame, app),
        Mode::Present => render_present(frame, app),
    }
}

fn render_browse(frame: &mut Frame, app: &App) {
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

    let current = app.current_slide();
    let content = render_slide_content(current.path.parent(), &current.raw_markdown);
    frame.render_widget(
        Paragraph::new(content)
            .block(
                Block::default()
                    .title(current.title.clone())
                    .borders(Borders::ALL),
            )
            .scroll((app.scroll, 0)),
        areas[1],
    );
}

fn render_present(frame: &mut Frame, app: &App) {
    let current = app.current_slide();
    let content = render_slide_content(current.path.parent(), &current.raw_markdown);
    frame.render_widget(Paragraph::new(content).scroll((app.scroll, 0)), frame.area());
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
    use super::render_slide_content;
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
    fn render_slide_content_includes_text_and_fallback_image_messages() {
        let dir = TempDir::new("ui-render");
        let content = render_slide_content(
            Some(dir.path()),
            "# Title\n\nBody\n\n![diagram](missing.png)",
        );

        assert!(content.contains("Title"));
        assert!(content.contains("Body"));
        assert!(content.contains("[missing image]"));
    }
}
