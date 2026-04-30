use anyhow::Result;
use notify_debouncer_mini::notify::RecursiveMode;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Duration;

type Debouncer =
    notify_debouncer_mini::Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>;

pub struct SlideWatcher {
    _debouncer: Debouncer,
    rx: Receiver<
        Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify_debouncer_mini::notify::Error>,
    >,
}

impl SlideWatcher {
    pub fn new(dir: &Path) -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = notify_debouncer_mini::new_debouncer(Duration::from_millis(200), tx)?;
        debouncer.watcher().watch(dir, RecursiveMode::Recursive)?;
        Ok(Self {
            _debouncer: debouncer,
            rx,
        })
    }

    /// Non-blocking check for .md file changes. Returns true if any qualifying
    /// event was found since the last call.
    pub fn poll_changes(&mut self) -> bool {
        let mut changed = false;
        for result in self.rx.try_iter() {
            match result {
                Ok(events) => {
                    for event in events {
                        if event.path.extension().is_some_and(|ext| ext == "md") {
                            changed = true;
                        }
                    }
                }
                Err(err) => {
                    eprintln!("[watcher] error: {err}");
                }
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::SlideWatcher;
    use std::fs;

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("slidet-watcher-{label}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn watcher_detects_new_md_file() {
        let dir = TempDir::new("detect-md");
        fs::write(dir.path().join("00-initial.md"), "# Hello").unwrap();

        let mut watcher = SlideWatcher::new(dir.path()).unwrap();

        // No changes yet
        assert!(!watcher.poll_changes());

        // Write a new .md file
        fs::write(dir.path().join("01-new.md"), "# New").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(350));

        assert!(watcher.poll_changes());
    }

    #[test]
    fn watcher_ignores_non_md_files() {
        let dir = TempDir::new("ignore-non-md");
        fs::write(dir.path().join("00-initial.md"), "# Hello").unwrap();

        let mut watcher = SlideWatcher::new(dir.path()).unwrap();

        // Wait for initial .md creation events to settle through debouncer
        std::thread::sleep(std::time::Duration::from_millis(350));
        watcher.poll_changes(); // drain all initial events

        // Write a non-.md file
        fs::write(dir.path().join("notes.txt"), "not a slide").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(350));

        assert!(!watcher.poll_changes());
    }
}
