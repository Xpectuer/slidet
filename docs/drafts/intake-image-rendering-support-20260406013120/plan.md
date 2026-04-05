---
title: "Plan: Markdown 幻灯片图片渲染"
doc_type: proc
brief: "为 slidet 制定 jpg/png 真渲染与 svg fallback 的实施步骤"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Plan: Markdown 幻灯片图片渲染

Spec: [spec.md](./spec.md)
Requirements: [requirements.md](./requirements.md)

## Files Changed

| File | Change Type |
|------|-------------|
| src/image.rs | Major edit |
| src/app.rs | Major edit |
| src/ui.rs | Major edit |
| examples/04-markdown-regression/03-image-and-fallback.md | Minor edit |

## Step 1 — 扩展图片准备与降级判定

**File**: `src/image.rs`  
**What**: 区分可渲染栅格图、`svg` 明确降级和缺失资源，并为三类结果补测试。

**Old** (exact surrounding text or heading anchor):
```rust
pub fn prepare_image(base_dir: &Path, src: &str) -> Result<ImageRender> {
    let resolved = base_dir.join(src);
    if !resolved.exists() {
        return Ok(ImageRender::FallbackText {
            message: format!("[missing image] {}", resolved.display()),
```

**New**:
```rust
pub fn prepare_image(base_dir: &Path, src: &str) -> Result<ImageRender> {
    let resolved = base_dir.join(src);
    if !resolved.exists() {
        return Ok(ImageRender::FallbackText {
            message: format!("[missing image] {}", resolved.display()),
        });
    }

    if is_svg(&resolved) {
        return Ok(ImageRender::FallbackText {
            message: format!("[svg unsupported] {} (svg 暂不支持渲染)", resolved.display()),
        });
    }

    if terminal_supports_images() {
        return Ok(ImageRender::TerminalImage { path: resolved });
    }

    Ok(ImageRender::FallbackText {
        message: format!("[image unavailable] {}", resolved.display()),
    })
}

#[test]
fn prepare_image_returns_terminal_image_for_png_assets_when_supported() { /* ... */ }

#[test]
fn prepare_image_returns_svg_fallback_for_existing_svg_assets() { /* ... */ }
```

**Verify**: `cargo test image::tests`

## Step 2 — 在线程安全边界内挂接图片渲染状态

**File**: `src/app.rs`  
**What**: 让 `App` 持有 `ratatui-image` 所需的 `Picker` / 图片状态缓存，并把 UI 渲染入口改为可变借用。

**Old** (exact surrounding text or heading anchor):
```rust
pub struct App {
    pub slides: Vec<Slide>,
    pub selected: usize,
    pub mode: Mode,
    pub scroll: u16,
    pub should_quit: bool,
}
```

**New**:
```rust
pub struct App {
    pub slides: Vec<Slide>,
    pub selected: usize,
    pub mode: Mode,
    pub scroll: u16,
    pub should_quit: bool,
    pub image_picker: Option<Picker>,
    pub image_states: HashMap<PathBuf, StatefulProtocol>,
}

impl App {
    pub fn image_state_for(&mut self, path: &Path) -> Result<&mut StatefulProtocol> { /* ... */ }
}

pub fn run(terminal: &mut DefaultTerminal, slides: Vec<Slide>) -> Result<()> {
    let mut app = App::new(slides);
    while !app.should_quit {
        terminal.draw(|frame| crate::ui::render(frame, &mut app))?;
        /* existing event loop */
    }
    Ok(())
}
```

**Verify**: `cargo test app::tests`

## Step 3 — 用真实图片 widget 替换字符串占位

**File**: `src/ui.rs`  
**What**: 将 slide 内容渲染改成按 Markdown 块顺序绘制文本或图片，在两种模式里共用 `ratatui-image` 的 `StatefulImage` + `Resize::Fit(None)` 链路。

**Old** (exact surrounding text or heading anchor):
```rust
pub fn render(frame: &mut Frame, app: &App) {
    match app.mode {
        Mode::Browse => render_browse(frame, app),
        Mode::Present => render_present(frame, app),
    }
}
```

**New**:
```rust
pub fn render(frame: &mut Frame, app: &mut App) {
    match app.mode {
        Mode::Browse => render_browse(frame, app),
        Mode::Present => render_present(frame, app),
    }
}

fn render_slide_blocks(frame: &mut Frame, area: Rect, app: &mut App, current: &Slide) {
    for block in markdown::parse_blocks(&current.raw_markdown) {
        match block {
            SlideBlock::Text(text) => frame.render_widget(Paragraph::new(text), text_area),
            SlideBlock::Image { src, .. } => match image::prepare_image(base, &src)? {
                ImageRender::TerminalImage { path } => {
                    let image = StatefulImage::default().resize(Resize::Fit(None));
                    frame.render_stateful_widget(image, image_area, app.image_state_for(&path)?);
                }
                ImageRender::FallbackText { message } => {
                    frame.render_widget(Paragraph::new(message), image_area);
                }
            },
        }
    }
}
```

**Verify**: `cargo test ui::tests`

## Step 4 — 更新回归样例以覆盖 PNG、SVG 与缺失资源

**File**: `examples/04-markdown-regression/03-image-and-fallback.md`  
**What**: 用同一页样例同时覆盖真实 PNG、已存在的 SVG fallback 和缺失资源 fallback，避免新增二进制 fixture。

**Old** (exact surrounding text or heading anchor):
```markdown
先看一张存在的图片资源：

![流程图资源](assets/render-path.svg)

再看一张故意缺失的图片：
```

**New**:
```markdown
先看一张存在的 PNG 图片资源：

![终端播放流程图](../02-image-demo/assets/terminal-flow.png)

再看一张已存在但暂不支持渲染的 SVG：

![流程图资源](assets/render-path.svg)

最后看一张故意缺失的图片：

![缺失资源](assets/not-found.png)
```

**Verify**: `rg -n "terminal-flow.png|render-path.svg|not-found.png" examples/04-markdown-regression/03-image-and-fallback.md`

## Step 5 — Proof-Read End-to-End

Read each changed file in full. Check: formatting, no leftover TODOs, spec intent preserved.

## Step 6 — Cross-Check Acceptance Criteria

| Criterion | Addressed in Step |
|-----------|------------------|
| Markdown 中的 `![alt](file.jpg)` 与 `![alt](file.png)` 在支持图片显示的终端中可实际显示为图片内容。 | Step 1, Step 2, Step 3 |
| `Browse` 模式与 `Present` 模式都遵循相同的图片加载与降级规则，不再只输出 `[image render] <path>`。 | Step 2, Step 3 |
| `svg` 资源在首版显示为明确文本占位，能让用户分辨“文件存在但格式暂不支持渲染”。 | Step 1, Step 3, Step 4 |
| 图片文件缺失时继续显示缺失提示；终端不支持图片时继续显示能力不足提示；两种情况都不得 panic。 | Step 1, Step 3 |
| 自动化测试覆盖至少三类情况：`jpg/png` 可渲染链路、`svg` fallback、缺失文件 fallback。 | Step 1, Step 3 |

## Step 7 — Review

Follow Phase 3 (see `03-self-review.md`). Writes `review.md`.

## Step 8 — Commit

Use `/commit`. Suggested message:
```text
docs: plan image rendering support
- write lean spec for raster image rendering
- add executable plan and self-review for implementation
- mark the intake draft ready for execution
```

## Execution Order

Step 1 → Step 2 → Step 3 → Step 4 → Step 5 → Step 6 → Step 7 → Step 8
