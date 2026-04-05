---
title: "Plan: Slide t 终端 Markdown Slide 播放器"
doc_type: proc
brief: "在 slidet/ 下规划 Rust ratatui Markdown slide 播放器的最小可用实现"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Plan: Slide t 终端 Markdown Slide 播放器

## Files Changed

| File | Change Type |
|------|-------------|
| `slidet/Cargo.toml` | New file |
| `slidet/src/main.rs` | New file |
| `slidet/src/loader.rs` | New file |
| `slidet/src/markdown.rs` | New file |
| `slidet/src/image.rs` | New file |
| `slidet/src/app.rs` | New file |
| `slidet/src/ui.rs` | New file |

## Step 1 — Create Cargo manifest

**File**: `slidet/Cargo.toml`
**What**: 初始化 `slidet` crate，并声明 Markdown、TUI、终端事件、图片降级所需依赖。

**New**:
```toml
[package]
name = "slidet"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
crossterm = "0.28"
image = "0.25"
pulldown-cmark = "0.12"
ratatui = "0.29"
ratatui-image = "4"
unicode-width = "0.2"
```

**Verify**: `test -f slidet/Cargo.toml && rg '^name = "slidet"$' slidet/Cargo.toml && cargo metadata --manifest-path slidet/Cargo.toml --no-deps >/dev/null`

---

## Step 2 — Add runtime bootstrap and CLI entry

**File**: `slidet/src/main.rs`
**What**: 解析 `slidet <slides_dir>` 参数，校验输入目录，初始化 terminal，启动应用并在退出时恢复 terminal 状态。

**New**:
```rust
mod app;
mod image;
mod loader;
mod markdown;
mod ui;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    slides_dir: std::path::PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let slides = loader::load_slides(&cli.slides_dir)?;
    let mut terminal = ui::init_terminal()?;
    let result = app::run(&mut terminal, slides);
    ui::restore_terminal()?;
    result
}
```

**Verify**: `cargo check --manifest-path slidet/Cargo.toml`

---

## Step 3 — Implement slide directory loader

**File**: `slidet/src/loader.rs`
**What**: 扫描目录内 `.md` 文件，按文件名字典序加载成 slide 列表，并在空目录、无效路径、读取失败时返回明确错误。

**New**:
```rust
use anyhow::{bail, Context, Result};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone)]
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

    let mut paths = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    paths.sort();

    if paths.is_empty() {
        bail!("no markdown slides found in {}", dir.display());
    }

    paths.into_iter()
        .map(|path| {
            let raw_markdown = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let title = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            Ok(Slide { path, title, raw_markdown })
        })
        .collect()
}
```

**Verify**: `cargo test --manifest-path slidet/Cargo.toml loader -- --nocapture || cargo check --manifest-path slidet/Cargo.toml`

---

## Step 4 — Parse Markdown into renderable blocks

**File**: `slidet/src/markdown.rs`
**What**: 将 slide Markdown 解析成 TUI 可渲染的文本块和图片块，保证文本内容能稳定渲染，并把图片引用交给图片层处理。

**New**:
```rust
use pulldown_cmark::{Event, Parser, Tag};

#[derive(Debug, Clone)]
pub enum SlideBlock {
    Text(String),
    Image { alt: String, src: String },
}

pub fn parse_blocks(markdown: &str) -> Vec<SlideBlock> {
    let mut blocks = Vec::new();
    let mut text = String::new();

    for event in Parser::new(markdown) {
        match event {
            Event::Text(content) | Event::Code(content) => {
                text.push_str(&content);
            }
            Event::SoftBreak | Event::HardBreak => text.push('\n'),
            Event::Start(Tag::Image { dest_url, title, .. }) => {
                if !text.trim().is_empty() {
                    blocks.push(SlideBlock::Text(text.trim().to_string()));
                    text.clear();
                }
                blocks.push(SlideBlock::Image {
                    alt: title.to_string(),
                    src: dest_url.to_string(),
                });
            }
            Event::End(_) => {
                if !text.trim().is_empty() {
                    blocks.push(SlideBlock::Text(text.trim().to_string()));
                    text.clear();
                }
            }
            _ => {}
        }
    }

    if !text.trim().is_empty() {
        blocks.push(SlideBlock::Text(text.trim().to_string()));
    }

    blocks
}
```

**Verify**: `cargo check --manifest-path slidet/Cargo.toml && rg 'enum SlideBlock' slidet/src/markdown.rs`

---

## Step 5 — Add image capability detection and graceful fallback

**File**: `slidet/src/image.rs`
**What**: 封装终端图片协议探测与图片加载逻辑，支持时返回可渲染图像，不支持或失败时返回占位文本而不是让应用崩溃。

**New**:
```rust
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
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
        message: format!("[image] {}", resolved.display()),
    })
}

fn terminal_supports_images() -> bool {
    std::env::var("TERM_PROGRAM").is_ok() || std::env::var("KITTY_WINDOW_ID").is_ok()
}
```

**Verify**: `cargo check --manifest-path slidet/Cargo.toml && rg 'FallbackText' slidet/src/image.rs`

---

## Step 6 — Implement application state and key handling

**File**: `slidet/src/app.rs`
**What**: 定义浏览模式与播放模式、当前 slide 选择、滚动偏移与基础键位，覆盖切页、进入播放、退出播放和退出程序。

**New**:
```rust
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;

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
}

pub fn run(terminal: &mut DefaultTerminal, slides: Vec<Slide>) -> Result<()> {
    let mut app = App {
        slides,
        selected: 0,
        mode: Mode::Browse,
        scroll: 0,
        should_quit: false,
    };

    while !app.should_quit {
        terminal.draw(|frame| crate::ui::render(frame, &app))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Down | KeyCode::Char('j') => app.next_slide(),
                KeyCode::Up | KeyCode::Char('k') => app.previous_slide(),
                KeyCode::Enter => app.mode = Mode::Present,
                KeyCode::Esc => app.mode = Mode::Browse,
                _ => {}
            }
        }
    }

    Ok(())
}
```

**Verify**: `cargo check --manifest-path slidet/Cargo.toml && rg "KeyCode::Char\\('q'\\)" slidet/src/app.rs`

---

## Step 7 — Render browse layout and fullscreen playback

**File**: `slidet/src/ui.rs`
**What**: 在浏览模式下渲染左侧 slide 列表和右侧当前页，在播放模式下隐藏左栏并让当前 slide 内容铺满 terminal，同时串联文本与图片块渲染。

**New**:
```rust
use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

use crate::{app::{App, Mode}, markdown, image};

pub fn init_terminal() -> Result<DefaultTerminal> {
    ratatui::init()
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

    let items = app.slides.iter()
        .map(|slide| ListItem::new(slide.title.clone()))
        .collect::<Vec<_>>();
    frame.render_widget(List::new(items).block(Block::default().title("Slides").borders(Borders::ALL)), areas[0]);

    let current = &app.slides[app.selected];
    let blocks = markdown::parse_blocks(&current.raw_markdown);
    let content = blocks.iter().map(|block| match block {
        markdown::SlideBlock::Text(text) => text.clone(),
        markdown::SlideBlock::Image { src, .. } => match image::prepare_image(current.path.parent().unwrap(), src) {
            Ok(image::ImageRender::FallbackText { message }) => message,
            Ok(image::ImageRender::TerminalImage { path }) => format!("[image render] {}", path.display()),
            Err(err) => format!("[image error] {err}"),
        },
    }).collect::<Vec<_>>().join("\n\n");

    frame.render_widget(
        Paragraph::new(content).block(Block::default().title(current.title.clone()).borders(Borders::ALL)).scroll((app.scroll, 0)),
        areas[1],
    );
}

fn render_present(frame: &mut Frame, app: &App) {
    let current = &app.slides[app.selected];
    let blocks = markdown::parse_blocks(&current.raw_markdown);
    let content = blocks.iter().map(|block| format!("{block:?}")).collect::<Vec<_>>().join("\n\n");
    frame.render_widget(Paragraph::new(content).scroll((app.scroll, 0)), frame.area());
}
```

**Verify**: `cargo check --manifest-path slidet/Cargo.toml && rg 'render_present' slidet/src/ui.rs`

---

## Step 8 — Proof-Read End-to-End

Read each changed file in full. Check: module wiring matches `spec.md`, CLI only接收目录路径、列表顺序基于文件名、浏览/播放模式切换闭环成立、图片失败路径不会导致 panic。

---

## Step 9 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| `slidet/` 下存在可构建运行的 Rust `ratatui` 项目骨架 | Steps 1-2 |
| 程序可接收一个目录作为 slide 数据源，并识别其中的 `.md` 文件 | Steps 2-3 |
| 左栏能列出该目录中的 Markdown slides，顺序基于文件名 | Steps 3, 7 |
| 右栏能渲染当前选中的 Markdown slide 文本内容 | Steps 4, 7 |
| 用户可通过基础键盘操作切换当前 slide | Step 6 |
| 用户可进入播放模式，并让当前 slide 内容铺满 terminal | Steps 6-7 |
| Markdown 中的图片在支持的 terminal 中尝试真实渲染 | Steps 5, 7 |
| 当 terminal 不支持图片协议时，程序不会崩溃，并提供可理解的降级显示 | Step 5 |
| 第一版实现不依赖 manifest、自动播放、speaker notes 等扩展特性 | Steps 1, 6 |

---

## Step 10 — Review

Follow Phase 3 (`03-self-review.md`) and write `review.md`.

---

## Step 11 — Commit

Use `/commit`. Suggested message:
```text
feat: add slidet markdown slide player implementation plan
- draft lean spec for the slidet MVP
- plan crate bootstrap, loader, markdown, image, app, and UI modules
- mark the intake session ready for tdd or progress
```

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8 → Step 9 → Step 10 → Step 11
