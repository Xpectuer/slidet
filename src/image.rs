use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageRender {
    TerminalImage { path: PathBuf },
    FallbackText { message: String },
}

pub fn prepare_image(base_dir: &Path, src: &str) -> Result<ImageRender> {
    let resolved = base_dir.join(src);
    if !resolved.exists() {
        return Ok(ImageRender::FallbackText {
            message: format!("[missing image] {}", resolved.display()),
        });
    }

    if terminal_supports_images() {
        return Ok(ImageRender::TerminalImage { path: resolved });
    }

    Ok(ImageRender::FallbackText {
        message: format!("[image unavailable] {}", resolved.display()),
    })
}

pub fn terminal_supports_images() -> bool {
    std::env::var_os("KITTY_WINDOW_ID").is_some()
        || matches!(std::env::var("TERM_PROGRAM").as_deref(), Ok("iTerm.app") | Ok("WezTerm"))
}

#[cfg(test)]
mod tests {
    use super::{prepare_image, ImageRender};
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
    fn prepare_image_returns_missing_message_for_missing_assets() {
        let dir = TempDir::new("image-missing");
        let render = prepare_image(dir.path(), "missing.png").unwrap();
        assert!(matches!(render, ImageRender::FallbackText { .. }));
    }
}
