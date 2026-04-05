---
title: "Image Module"
doc_type: module
module_name: image
module_path: src/image.rs
generated_by: mci-phase-2
brief: "Terminal image capability detection with graceful fallback strategies"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Image Module

## Overview

The image module provides terminal image rendering capability detection and graceful degradation strategies. It ensures the application never crashes due to unsupported image formats or missing files, instead providing informative fallback text.

<!-- BEGIN:Interface -->
## Interface

### Public Types

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageRender {
    TerminalImage { path: PathBuf },     // Terminal supports graphics protocol
    FallbackText { message: String },    // Graceful degradation message
}
```

### Public Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `prepare_image` | `fn(base_dir: &Path, src: &str) -> Result<ImageRender>` | Resolves image path, checks existence, handles SVG fallback, returns appropriate render strategy |
| `terminal_supports_images` | `fn() -> bool` | Detects if current terminal supports inline graphics protocols |

### Internal Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `is_svg` | `fn(path: &Path) -> bool` | Checks file extension for SVG (case-insensitive) |
<!-- END:Interface -->

<!-- BEGIN:DependencyGraph -->
## Dependency Graph

```
src/image.rs
    |
    +-- std::path::{Path, PathBuf}  -- Path manipulation
    |
    +-- anyhow::Result              -- Error handling
```

### Downstream Consumers

| Module | Usage |
|--------|-------|
| `src/markdown.rs` | Converts inline images to `SlideBlock::Image` |
| `src/ui.rs` | Renders `ImageRender` to terminal (graphics or fallback text) |
<!-- END:DependencyGraph -->

<!-- BEGIN:StateManagement -->
## State Management

The image module is **stateless**. All functions are pure and deterministic:

- `prepare_image`: Deterministic based on filesystem state and environment variables
- `terminal_supports_images`: Reads process environment, no mutation
- `is_svg`: Pure function of path extension

No mutable static state or interior mutability is used.
<!-- END:StateManagement -->

<!-- BEGIN:EdgeCases -->
## Edge Cases

### Supported Terminals (Hardcoded)

The module detects graphics support by checking environment variables:

| Environment Variable | Terminal | Protocol |
|---------------------|----------|----------|
| `KITTY_WINDOW_ID` | Kitty | Kitty Graphics Protocol |
| `TERM_PROGRAM=iTerm.app` | iTerm2 | Inline Images Protocol |
| `TERM_PROGRAM=WezTerm` | WezTerm | iTerm2 Protocol |
| `TERM_PROGRAM=ghostty` | Ghostty | Kitty Graphics Protocol |

### Fallback Scenarios

| Scenario | Output Format |
|----------|---------------|
| File not found | `[missing image] /full/path/to/file.png` |
| SVG file | `[svg unsupported] /full/path/to/file.svg (svg 暂不支持渲染)` |
| Unsupported terminal | `[image unavailable] /full/path/to/file.png` |

### Error Handling Strategy

The module uses `anyhow::Result` but **never returns errors** to callers:

1. Missing files → `FallbackText` with `[missing image]` prefix
2. SVG format → `FallbackText` with `[svg unsupported]` prefix
3. Unsupported terminal → `FallbackText` with `[image unavailable]` prefix

This ensures graceful degradation without propagating errors up the call stack.
<!-- END:EdgeCases -->

<!-- BEGIN:UsageExample -->
## Usage Example

```rust
use std::path::Path;
use slidet::image::{prepare_image, terminal_supports_images, ImageRender};

fn render_slide_image(base_dir: &Path, image_src: &str) {
    match prepare_image(base_dir, image_src).unwrap() {
        ImageRender::TerminalImage { path } => {
            // Terminal supports graphics - use protocol-specific rendering
            println!("Rendering image: {}", path.display());
            // In actual code: call terminal graphics protocol
        }
        ImageRender::FallbackText { message } => {
            // Graceful degradation - show text placeholder
            println!("{}", message);
        }
    }
}

fn check_environment() {
    if terminal_supports_images() {
        println!("Terminal supports inline images");
    } else {
        println!("Images will be shown as text placeholders");
    }
}
```

### Integration with Markdown Pipeline

```rust
// In markdown.rs parsing
fn parse_image(alt: &str, src: &str) -> SlideBlock {
    SlideBlock::Image {
        alt: alt.to_string(),
        src: src.to_string(),
    }
}

// In ui.rs rendering
fn render_image_block(base_dir: &Path, src: &str) -> Text<'static> {
    match prepare_image(base_dir, src).unwrap() {
        ImageRender::TerminalImage { path } => {
            // Use ratatui-image or graphics protocol
            render_terminal_image(&path)
        }
        ImageRender::FallbackText { message } => {
            Text::styled(message, Style::default().fg(Color::Yellow))
        }
    }
}
```
<!-- END:UsageExample -->
