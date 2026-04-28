use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::loader::Slide;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Present,
}

pub struct App {
    pub slides: Vec<Slide>,
    pub selected: usize,
    pub mode: Mode,
    pub scroll: u16,
    pub should_quit: bool,
    pub image: ImageContext,
    pub slides_dir: PathBuf,
    pub watcher: Option<crate::watcher::SlideWatcher>,
    pub reload_indicator: Option<Instant>,
}

pub struct ImageContext {
    pub image_picker: Option<Picker>,
    pub image_states: HashMap<PathBuf, StatefulProtocol>,
}

impl App {
    pub fn new(slides: Vec<Slide>, slides_dir: PathBuf) -> Self {
        Self::with_image_picker(slides, slides_dir, default_image_picker())
    }

    fn with_image_picker(
        slides: Vec<Slide>,
        slides_dir: PathBuf,
        image_picker: Option<Picker>,
    ) -> Self {
        let watcher = crate::watcher::SlideWatcher::new(&slides_dir)
            .map_err(|e| eprintln!("[watcher] file watching unavailable: {e}"))
            .ok();
        Self {
            slides,
            slides_dir,
            selected: 0,
            mode: Mode::Browse,
            scroll: 0,
            should_quit: false,
            image: ImageContext {
                image_picker,
                image_states: HashMap::new(),
            },
            watcher,
            reload_indicator: None,
        }
    }

    pub fn current_slide(&self) -> &Slide {
        &self.slides[self.selected]
    }

    pub fn next_slide(&mut self) {
        if self.selected + 1 < self.slides.len() {
            self.selected += 1;
            self.scroll = 0;
        }
    }

    pub fn previous_slide(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.scroll = 0;
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => self.next_slide(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_slide(),
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(5),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(5),
            KeyCode::Enter => {
                self.mode = Mode::Present;
                self.scroll = 0;
            }
            KeyCode::Esc => {
                self.mode = Mode::Browse;
                self.scroll = 0;
            }
            _ => {}
        }
    }

    pub fn reload_slides(&mut self) {
        let current_path = self.current_slide().path.clone();

        match crate::loader::load_slides(&self.slides_dir) {
            Ok(new_slides) => {
                self.slides = new_slides;
                self.image.image_states.clear();

                // Try to keep the user on the same slide by path
                if let Some(idx) = self.slides.iter().position(|s| s.path == current_path) {
                    self.selected = idx;
                } else {
                    self.selected = self.selected.min(self.slides.len().saturating_sub(1));
                }
                self.scroll = 0;
                self.reload_indicator = Some(Instant::now());
            }
            Err(e) => {
                // Keep showing old slides — the directory may be temporarily empty
                eprintln!("[watcher] reload failed: {e}");
            }
        }
    }

    pub fn image_state_for(&mut self, path: &Path) -> Result<&mut StatefulProtocol> {
        self.image.image_state_for(path)
    }
}

impl ImageContext {
    pub fn image_state_for(&mut self, path: &Path) -> Result<&mut StatefulProtocol> {
        let cache_key = path.to_path_buf();
        if !self.image_states.contains_key(&cache_key) {
            let picker = self
                .image_picker
                .as_ref()
                .context("image rendering is unavailable for this terminal")?;
            let dyn_img = image::open(path)
                .with_context(|| format!("failed to decode image {}", path.display()))?;
            let protocol = picker.new_resize_protocol(dyn_img);
            self.image_states.insert(cache_key.clone(), protocol);
        }

        self.image_states
            .get_mut(&cache_key)
            .context("missing cached image state after initialization")
    }
}

impl crate::ui::ImageStateStore for ImageContext {
    fn image_state_for(&mut self, path: &Path) -> Result<&mut StatefulProtocol> {
        ImageContext::image_state_for(self, path)
    }
}

pub fn run(terminal: &mut DefaultTerminal, slides: Vec<Slide>, slides_dir: PathBuf) -> Result<()> {
    let mut app = App::new(slides, slides_dir);

    while !app.should_quit {
        terminal.draw(|frame| {
            let (slides, selected, mode, scroll, image, reload_indicator) = (
                &app.slides,
                app.selected,
                app.mode,
                app.scroll,
                &mut app.image,
                app.reload_indicator,
            );
            let model = crate::ui::RenderModel {
                slides,
                selected,
                mode: match mode {
                    Mode::Browse => crate::ui::RenderMode::Browse,
                    Mode::Present => crate::ui::RenderMode::Present,
                },
                scroll,
            };
            crate::ui::render(frame, &model, image);

            if let Some(instant) = reload_indicator {
                if instant.elapsed().as_secs() < 2 {
                    crate::ui::render_reload_indicator(frame);
                }
            }
        })?;

        // Poll for terminal events with a 100ms timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        // Check for file changes (non-blocking)
        if let Some(ref mut watcher) = app.watcher {
            if watcher.poll_changes() {
                app.reload_slides();
            }
        }

        // Expire reload indicator
        if let Some(instant) = app.reload_indicator {
            if instant.elapsed().as_secs() >= 2 {
                app.reload_indicator = None;
            }
        }
    }

    Ok(())
}

fn default_image_picker() -> Option<Picker> {
    if !crate::image::terminal_supports_images() {
        return None;
    }

    Some(Picker::from_query_stdio().unwrap_or_else(|_| Picker::from_fontsize((10, 20))))
}

#[cfg(test)]
mod tests {
    use super::{App, Mode};
    use crate::loader::Slide;
    use crossterm::event::KeyCode;
    use ratatui_image::picker::Picker;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn slides() -> Vec<Slide> {
        vec![
            Slide {
                path: PathBuf::from("01.md"),
                title: String::from("01"),
                raw_markdown: String::from("# One"),
            },
            Slide {
                path: PathBuf::from("02.md"),
                title: String::from("02"),
                raw_markdown: String::from("# Two"),
            },
        ]
    }

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
    fn app_navigates_and_switches_modes() {
        let mut app = App::new(slides(), PathBuf::from("/tmp/test-slides"));

        app.handle_key(KeyCode::Down);
        assert_eq!(app.selected, 1);

        app.handle_key(KeyCode::Enter);
        assert_eq!(app.mode, Mode::Present);

        app.handle_key(KeyCode::Esc);
        assert_eq!(app.mode, Mode::Browse);

        app.handle_key(KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn image_state_for_caches_render_state_per_path() {
        let dir = TempDir::new("app-image-state");
        let image_path = dir.path().join("photo.png");
        image::DynamicImage::new_rgba8(1, 1)
            .save(&image_path)
            .unwrap();

        let mut app = App::with_image_picker(slides(), PathBuf::from("/tmp/test-slides"), Some(Picker::from_fontsize((8, 16))));

        app.image_state_for(&image_path).unwrap();
        app.image_state_for(&image_path).unwrap();

        assert_eq!(app.image.image_states.len(), 1);
    }
}
