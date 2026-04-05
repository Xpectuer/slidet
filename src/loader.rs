use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slide {
    pub path: PathBuf,
    pub title: String,
    pub raw_markdown: String,
}

pub fn load_slides(dir: &Path) -> Result<Vec<Slide>> {
    if !dir.exists() {
        bail!("slides directory does not exist: {}", dir.display());
    }
    if !dir.is_dir() {
        bail!("slides path is not a directory: {}", dir.display());
    }

    let mut paths = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to enumerate {}", dir.display()))?;

    paths.retain(|path| path.extension().and_then(|s| s.to_str()) == Some("md"));
    paths.sort();

    if paths.is_empty() {
        bail!("no markdown slides found in {}", dir.display());
    }

    paths
        .into_iter()
        .map(|path| {
            let raw_markdown = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let title = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| String::from("untitled"));

            Ok(Slide {
                path,
                title,
                raw_markdown,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::load_slides;
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
    fn load_slides_sorts_markdown_files_by_name() {
        let dir = TempDir::new("loader-order");
        fs::write(dir.path().join("02-agenda.md"), "## Agenda").unwrap();
        fs::write(dir.path().join("01-intro.md"), "# Intro").unwrap();
        fs::write(dir.path().join("notes.txt"), "ignored").unwrap();

        let slides = load_slides(dir.path()).unwrap();

        assert_eq!(slides.len(), 2);
        assert_eq!(slides[0].title, "01-intro");
        assert_eq!(slides[1].title, "02-agenda");
        assert_eq!(slides[0].raw_markdown, "# Intro");
    }

    #[test]
    fn load_slides_errors_for_missing_directory() {
        let err = load_slides(Path::new("/definitely/missing/slides"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("slides directory does not exist"));
    }

    #[test]
    fn load_slides_errors_for_non_directory_paths() {
        let dir = TempDir::new("loader-file");
        let file = dir.path().join("slides.md");
        fs::write(&file, "# Slide").unwrap();

        let err = load_slides(&file).unwrap_err().to_string();
        assert!(err.contains("slides path is not a directory"));
    }

    #[test]
    fn load_slides_errors_for_empty_directory() {
        let dir = TempDir::new("loader-empty");

        let err = load_slides(dir.path()).unwrap_err().to_string();
        assert!(err.contains("no markdown slides found"));
    }
}
