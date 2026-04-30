use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::loader::{Slide, SlideNode, VisibleItem, VisibleItemKind, compute_visible_items};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Present,
}

pub struct App {
    pub nodes: Vec<SlideNode>,
    pub visible: Vec<VisibleItem>,
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
    pub fn new(nodes: Vec<SlideNode>, slides_dir: PathBuf) -> Self {
        Self::with_image_picker(nodes, slides_dir, default_image_picker())
    }

    fn with_image_picker(
        nodes: Vec<SlideNode>,
        slides_dir: PathBuf,
        image_picker: Option<Picker>,
    ) -> Self {
        let watcher = crate::watcher::SlideWatcher::new(&slides_dir)
            .map_err(|e| eprintln!("[watcher] file watching unavailable: {e}"))
            .ok();
        let visible = compute_visible_items(&nodes);
        Self {
            nodes,
            visible,
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

    pub fn current_slide(&self) -> Option<&Slide> {
        let item = self.visible.get(self.selected)?;
        let slide_ref = item.slide_ref.as_ref()?;
        Some(SlideNode::resolve_slide(&self.nodes, slide_ref))
    }

    pub fn next_slide(&mut self) {
        if self.selected + 1 < self.visible.len() {
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

    fn next_slide_present(&mut self) {
        let start = self.selected + 1;
        for i in start..self.visible.len() {
            if self.visible[i].slide_ref.is_some() {
                self.selected = i;
                self.scroll = 0;
                return;
            }
        }
    }

    fn previous_slide_present(&mut self) {
        if self.selected == 0 {
            return;
        }
        for i in (0..self.selected).rev() {
            if self.visible[i].slide_ref.is_some() {
                self.selected = i;
                self.scroll = 0;
                return;
            }
        }
    }

    pub fn toggle_expand(&mut self) {
        let group_index = match self.visible.get(self.selected) {
            Some(item) if matches!(item.kind, VisibleItemKind::Group { .. }) => item.group_index,
            _ => return,
        };
        if let SlideNode::Group { ref mut expanded, .. } = &mut self.nodes[group_index] {
            *expanded = !*expanded;
            self.visible = compute_visible_items(&self.nodes);
            self.selected = self.selected.min(self.visible.len().saturating_sub(1));
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.mode == Mode::Present {
                    self.next_slide_present();
                } else {
                    self.next_slide();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.mode == Mode::Present {
                    self.previous_slide_present();
                } else {
                    self.previous_slide();
                }
            }
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(5),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(5),
            KeyCode::Enter => {
                if self.mode == Mode::Present {
                    self.mode = Mode::Browse;
                    self.scroll = 0;
                } else if self.selected < self.visible.len() {
                    if matches!(self.visible[self.selected].kind, VisibleItemKind::Group { .. }) {
                        self.toggle_expand();
                    } else {
                        self.mode = Mode::Present;
                        self.scroll = 0;
                    }
                }
            }
            KeyCode::Esc => {
                self.mode = Mode::Browse;
                self.scroll = 0;
            }
            _ => {}
        }
    }

    pub fn reload_slides(&mut self) {
        let current_path = self.current_slide().map(|s| s.path.clone());

        match crate::loader::load_slides(&self.slides_dir) {
            Ok(new_nodes) => {
                // Preserve expand state by matching group paths
                let old_expand: HashMap<PathBuf, bool> = self.nodes.iter().filter_map(|n| {
                    if let SlideNode::Group { path, expanded, .. } = n {
                        Some((path.clone(), *expanded))
                    } else {
                        None
                    }
                }).collect();

                self.nodes = new_nodes;
                // Restore expand state
                for node in &mut self.nodes {
                    if let SlideNode::Group { path, ref mut expanded, .. } = node {
                        if let Some(was_expanded) = old_expand.get(path) {
                            *expanded = *was_expanded;
                        }
                    }
                }

                self.visible = compute_visible_items(&self.nodes);
                self.image.image_states.clear();

                // Try to keep the user on the same slide by path
                if let Some(ref path) = current_path {
                    if let Some(idx) = self.visible.iter().position(|item| {
                        item.slide_ref.as_ref().is_some_and(|r| {
                            SlideNode::resolve_slide(&self.nodes, r).path == *path
                        })
                    }) {
                        self.selected = idx;
                    } else {
                        self.selected = self.selected.min(self.visible.len().saturating_sub(1));
                    }
                } else {
                    self.selected = self.selected.min(self.visible.len().saturating_sub(1));
                }
                self.scroll = 0;
                self.reload_indicator = Some(Instant::now());
            }
            Err(e) => {
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

pub fn run(terminal: &mut DefaultTerminal, nodes: Vec<SlideNode>, slides_dir: PathBuf) -> Result<()> {
    let mut app = App::new(nodes, slides_dir);

    while !app.should_quit {
        terminal.draw(|frame| {
            let (nodes, visible, selected, mode, scroll, image, reload_indicator) = (
                &app.nodes,
                &app.visible,
                app.selected,
                app.mode,
                app.scroll,
                &mut app.image,
                app.reload_indicator,
            );
            let model = crate::ui::RenderModel {
                nodes,
                visible,
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
    use crate::loader::{Slide, SlideNode};
    use crossterm::event::KeyCode;
    use ratatui_image::picker::Picker;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn leaf_nodes() -> Vec<SlideNode> {
        vec![
            SlideNode::Leaf(Slide {
                path: PathBuf::from("01.md"),
                title: String::from("01"),
                raw_markdown: String::from("# One"),
            }),
            SlideNode::Leaf(Slide {
                path: PathBuf::from("02.md"),
                title: String::from("02"),
                raw_markdown: String::from("# Two"),
            }),
        ]
    }

    fn nodes_with_group() -> Vec<SlideNode> {
        vec![
            SlideNode::Leaf(Slide {
                path: PathBuf::from("00-intro.md"),
                title: String::from("00-intro"),
                raw_markdown: String::from("# Intro"),
            }),
            SlideNode::Group {
                name: String::from("01-topic"),
                path: PathBuf::from("01-topic"),
                children: vec![
                    Slide {
                        path: PathBuf::from("01-topic/01-a.md"),
                        title: String::from("01-a"),
                        raw_markdown: String::from("# A"),
                    },
                    Slide {
                        path: PathBuf::from("01-topic/02-b.md"),
                        title: String::from("02-b"),
                        raw_markdown: String::from("# B"),
                    },
                ],
                expanded: false,
            },
            SlideNode::Leaf(Slide {
                path: PathBuf::from("02-summary.md"),
                title: String::from("02-summary"),
                raw_markdown: String::from("# Summary"),
            }),
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
    fn app_navigates_flat_leaves() {
        let mut app = App::new(leaf_nodes(), PathBuf::from("/tmp/test-slides"));

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
    fn app_toggle_expand_collapse() {
        let mut app = App::new(nodes_with_group(), PathBuf::from("/tmp/test-slides"));
        // 0: intro leaf, 1: group (collapsed), 2: summary leaf
        assert_eq!(app.visible.len(), 3);

        // Navigate to group and expand
        app.handle_key(KeyCode::Down);
        assert_eq!(app.selected, 1);
        app.handle_key(KeyCode::Enter);
        // Now group is expanded: intro, group, child-a, child-b, summary
        assert_eq!(app.visible.len(), 5);

        // Collapse
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.visible.len(), 3);
    }

    #[test]
    fn app_present_mode_skips_groups() {
        let mut nodes = nodes_with_group();
        // Expand the group first
        if let SlideNode::Group { ref mut expanded, .. } = nodes[1] {
            *expanded = true;
        }
        let mut app = App::new(nodes, PathBuf::from("/tmp/test-slides"));
        // visible: intro(0), group(1), child-a(2), child-b(3), summary(4)

        app.handle_key(KeyCode::Enter); // on intro leaf -> Present mode
        assert_eq!(app.mode, Mode::Present);

        // In Present mode, j should skip group header and go to child-a
        app.handle_key(KeyCode::Down);
        assert!(app.current_slide().unwrap().title == "01-a");

        app.handle_key(KeyCode::Down);
        assert!(app.current_slide().unwrap().title == "02-b");

        app.handle_key(KeyCode::Down);
        assert!(app.current_slide().unwrap().title == "02-summary");
    }

    #[test]
    fn image_state_for_caches_render_state_per_path() {
        let dir = TempDir::new("app-image-state");
        let image_path = dir.path().join("photo.png");
        image::DynamicImage::new_rgba8(1, 1)
            .save(&image_path)
            .unwrap();

        let mut app = App::with_image_picker(leaf_nodes(), PathBuf::from("/tmp/test-slides"), Some(Picker::from_fontsize((8, 16))));

        app.image_state_for(&image_path).unwrap();
        app.image_state_for(&image_path).unwrap();

        assert_eq!(app.image.image_states.len(), 1);
    }
}
