use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::loader::Slide;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Present,
}

#[derive(Debug)]
pub struct App {
    pub slides: Vec<Slide>,
    pub selected: usize,
    pub mode: Mode,
    pub scroll: u16,
    pub should_quit: bool,
}

impl App {
    pub fn new(slides: Vec<Slide>) -> Self {
        Self {
            slides,
            selected: 0,
            mode: Mode::Browse,
            scroll: 0,
            should_quit: false,
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
}

pub fn run(terminal: &mut DefaultTerminal, slides: Vec<Slide>) -> Result<()> {
    let mut app = App::new(slides);

    while !app.should_quit {
        terminal.draw(|frame| crate::ui::render(frame, &app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{App, Mode};
    use crate::loader::Slide;
    use crossterm::event::KeyCode;
    use std::path::PathBuf;

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

    #[test]
    fn app_navigates_and_switches_modes() {
        let mut app = App::new(slides());

        app.handle_key(KeyCode::Down);
        assert_eq!(app.selected, 1);

        app.handle_key(KeyCode::Enter);
        assert_eq!(app.mode, Mode::Present);

        app.handle_key(KeyCode::Esc);
        assert_eq!(app.mode, Mode::Browse);

        app.handle_key(KeyCode::Char('q'));
        assert!(app.should_quit);
    }
}
