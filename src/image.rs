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

    if is_svg(&resolved) {
        return Ok(ImageRender::FallbackText {
            message: format!(
                "[svg unsupported] {} (svg 暂不支持渲染)",
                resolved.display()
            ),
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
        || matches!(
            std::env::var("TERM_PROGRAM").as_deref(),
            Ok("iTerm.app") | Ok("WezTerm") | Ok("ghostty")
        )
}

fn is_svg(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
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

    #[test]
    fn prepare_image_returns_svg_fallback_for_existing_svg_assets() {
        let dir = TempDir::new("image-svg");
        fs::write(dir.path().join("diagram.svg"), "<svg></svg>").unwrap();

        let render = prepare_image(dir.path(), "diagram.svg").unwrap();

        assert_eq!(
            render,
            ImageRender::FallbackText {
                message: format!(
                    "[svg unsupported] {} (svg 暂不支持渲染)",
                    dir.path().join("diagram.svg").display()
                ),
            }
        );
    }

    #[test]
    fn prepare_image_returns_terminal_image_for_png_assets_when_supported() {
        let dir = TempDir::new("image-png");
        fs::write(dir.path().join("photo.png"), b"png").unwrap();
        let previous = std::env::var_os("KITTY_WINDOW_ID");
        std::env::set_var("KITTY_WINDOW_ID", "test-window");

        let render = prepare_image(dir.path(), "photo.png").unwrap();

        match previous {
            Some(value) => std::env::set_var("KITTY_WINDOW_ID", value),
            None => std::env::remove_var("KITTY_WINDOW_ID"),
        }

        assert_eq!(
            render,
            ImageRender::TerminalImage {
                path: dir.path().join("photo.png"),
            }
        );
    }

    #[test]
    fn terminal_supports_images_accepts_ghostty() {
        let previous_kitty = std::env::var_os("KITTY_WINDOW_ID");
        let previous_term_program = std::env::var_os("TERM_PROGRAM");
        std::env::remove_var("KITTY_WINDOW_ID");
        std::env::set_var("TERM_PROGRAM", "ghostty");

        let supported = super::terminal_supports_images();

        match previous_kitty {
            Some(value) => std::env::set_var("KITTY_WINDOW_ID", value),
            None => std::env::remove_var("KITTY_WINDOW_ID"),
        }
        match previous_term_program {
            Some(value) => std::env::set_var("TERM_PROGRAM", value),
            None => std::env::remove_var("TERM_PROGRAM"),
        }

        assert!(supported);
    }
}
