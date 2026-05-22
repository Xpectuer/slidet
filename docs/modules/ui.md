---
doc_type: module
module_name: ui
module_path: src/ui.rs
generated_by: mci-phase-2
created: 2026-04-06
updated: 2026-05-23
revision: 4
---

# ui Module

UI 渲染模块，负责终端初始化/恢复、Browse/Present 双模式视图渲染、Markdown 文本渲染和图片渲染。

<!-- BEGIN:INTERFACE -->

## Interface

模块公开以下函数：

| 函数 | 签名 | 说明 |
|------|------|------|
| `init_terminal` | `fn() -> Result<DefaultTerminal>` | 初始化 ratatui 终端，返回可用于渲染的 Terminal 实例 |
| `restore_terminal` | `fn() -> Result<()>` | 恢复终端到原始状态，程序退出前必须调用 |
| `render` | `fn(frame: &mut Frame, model: &RenderModel, image_states: &mut dyn ImageStateStore)` | 主渲染入口，根据 `RenderMode` 分发到 Browse 或 Present 模式 |
| `render_reload_indicator` | `fn(frame: &mut Frame)` | 在帧右下角渲染绿色 "Reloaded" 标签（由 app 层控制显示时机） |
| `render_slide_content` | `fn(base_dir: Option<&Path>, raw_markdown: &str) -> String` | 将 Markdown 内容渲染为纯文本字符串（用于调试/测试） |

### 公开类型

| 类型 | 说明 |
|------|------|
| `RenderMode` | UI 使用的显示模式，和 app 层的 `Mode` 解耦 |
| `RenderModel<'a>` | UI 只读渲染模型，包含 slides、selected、mode、scroll |
| `ImageStateStore` | 图片状态访问接口，允许 UI 使用外部图片缓存而不依赖 `App` |

### 内部渲染函数

| 函数 | 说明 |
|------|------|
| `render_browse` | Browse 模式：左侧 slide 列表 + 右侧预览区域 |
| `render_present` | Present 模式：全屏展示当前 slide |
| `render_slide_blocks` | 渲染 slide 的所有块（文本 + 图片），处理滚动裁剪 |
| `render_markdown_block` | 渲染单个 Markdown 块到 Paragraph widget |
| `render_image_block` | 渲染图片块，支持终端图片和 fallback 文本 |
| `render_reload_indicator` | 渲染右下角绿色 "Reloaded" 标签（宽度: text.len + 2，高度: 1 行） |

<!-- END:INTERFACE -->

<!-- BEGIN:DEPENDENCIES -->

## Dependency Graph

```
ui.rs
├── External Crates
│   ├── ratatui (Terminal UI framework)
│   │   ├── layout::{Alignment, Constraint, Direction, Layout, Rect}
│   │   ├── style::{Color, Modifier, Style}
│   │   ├── text::{Line, Span, Text}
│   │   ├── widgets::{Block, Borders, Paragraph, Wrap}
│   │   └── {DefaultTerminal, Frame}
│   └── ratatui_image (Image rendering)
│       └── {Resize, StatefulImage}
│
└── Internal Modules
    ├── image::{self, ImageRender}-- 图片准备和渲染类型
    ├── loader::Slide             -- Slide 数据结构
    └── markdown::{self, SlideBlock, MarkdownBlock, InlineSpan, ListItem}
                                   -- Markdown 解析结果类型
```

### 数据流向

```
RenderModel (state view)
    │
    ▼
render() ──┬── RenderMode::Browse ──► render_browse()
           │                         │
           │                         ├── Layout 分割为 [nav_area, preview_area]
           │                         └── render_slide_blocks()
           │
           └── RenderMode::Present ─► render_present()
                                     │
                                     └── render_slide_blocks()
                                              │
                                              ├── SlideBlock::Markdown ──► render_markdown_block()
                                              │                                    │
                                              │                                    └── Paragraph widget
                                              │
                                              └── SlideBlock::Image ──► render_image_block()
                                                                           │
                                                                           ├── ImageRender::TerminalImage ──► StatefulImage widget
                                                                           └── ImageRender::FallbackText ──► Paragraph widget
```

<!-- END:DEPENDENCIES -->

<!-- BEGIN:STATE_MANAGEMENT -->

## State Management

### RenderModel 状态依赖

`ui` 模块不维护自己的状态，而是读取 `RenderModel` 并通过 `ImageStateStore` 访问图片缓存：

| 来源 | 读/写 | 用途 |
|------|-------|------|
| `RenderModel.mode` | 读 | 决定渲染 Browse 还是 Present 视图 |
| `RenderModel.slides` | 读 | 获取所有 slide 列表用于导航面板 |
| `RenderModel.selected` | 读 | 当前选中的 slide 索引 |
| `RenderModel.scroll` | 读 | 垂直滚动偏移量 |
| `ImageStateStore` | 读写 | 获取或创建图片渲染状态缓存 |

### 图片状态缓存

`ImageStateStore::image_state_for(&path)` 方法用于获取或创建图片渲染状态：

- 首次访问时创建新的 `ImageState` 并缓存
- 后续访问复用已缓存的状态
- 状态用于 `StatefulImage` widget 的增量渲染

### 滚动机制

滚动通过 `RenderModel.scroll` 字段控制：

1. `render_slide_blocks` 计算 `cursor_y = area.y - model.scroll`
2. `clip_rect()` 根据可见区域裁剪每个块
3. 对于部分可见的文本块，返回 `text_scroll` 值传递给 `Paragraph::scroll()`

<!-- END:STATE_MANAGEMENT -->

<!-- BEGIN:EDGE_CASES -->

## Edge Cases

### 硬编码值

| 位置 | 值 | 说明 |
|------|-----|------|
| `render_browse()` | `Constraint::Length(28)` | 左侧导航面板固定宽度 28 字符 |
| `block_height()` | `12` | 图片块默认高度 12 行 |
| `push_list_item_lines()` | `- [x] ` / `- [ ] ` / `- ` | 列表项前缀格式 |
| `render_markdown_block()` | `    ` (4 spaces) | 代码块缩进 |
| `heading_style()` | `Color::LightYellow` | 标题颜色 |
| `render_inline_span()` | `Color::Green` / `Color::DarkGray` | 行内代码颜色 |
| `render_inline_span()` | `Color::LightBlue` + `Modifier::UNDERLINED` | 链接标签颜色和样式 |
| `render_inline_span()` | `Color::DarkGray` | 链接 URL 颜色（让终端自动检测为可点击链接，避开 OSC 8 的 ratatui 不兼容问题） |
| `render_reload_indicator()` | `Color::Green` + `Modifier::BOLD` | 重载指示器文字样式 |
| `render_reload_indicator()` | `width + 2`, `height - 2` | 右下角偏移位置 |

### 边界条件处理

| 条件 | 处理方式 |
|------|----------|
| `area.width == 0 \|\| area.height == 0` | `render_slide_blocks` 直接返回，不渲染 |
| `bottom <= area_top \|\| top >= area_bottom` | `clip_rect` 返回 `None`，跳过该块 |
| `lines.is_empty()` | `render_markdown_text` 插入空行 `Line::default()` |
| `width.max(1)` | `estimate_text_height` 确保宽度至少为 1 |
| `image::prepare_image()` 失败 | 显示 `[image error] {err}` 占位文本 |
| `image_states.image_state_for()` 失败 | 显示 `[image error] {err}` 占位文本 |
| `path.parent()` 返回 None | 使用 `.` 作为 base_dir |

### 错误处理策略

- **图片加载失败**: 降级为 `[image error] {message}` 文本显示
- **终端不支持图片**: 通过 `image::prepare_image()` 返回 `FallbackText`
- **Markdown 解析错误**: 由 `markdown` 模块处理，ui 层假设输入有效
- **终端初始化/恢复失败**: 返回 `anyhow::Result`，由调用方处理

<!-- END:EDGE_CASES -->

<!-- BEGIN:USAGE_EXAMPLE -->

## Usage Example

```rust
use slidet::app::{App, ImageContext};
use slidet::ui::{init_terminal, restore_terminal, render};
use slidet::loader::Slide;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // 1. 初始化终端
    let mut terminal = init_terminal()?;

    // 2. 创建应用状态
    let slides = vec![
        Slide {
            path: PathBuf::from("slides/01-intro.md"),
            title: String::from("Introduction"),
            raw_markdown: String::from("# Welcome\n\nThis is the first slide."),
        },
        Slide {
            path: PathBuf::from("slides/02-demo.md"),
            title: String::from("Demo"),
            raw_markdown: String::from("# Demo\n\n![screenshot](demo.png)"),
        },
    ];

    let mut app = App {
        slides,
        selected: 0,
        mode: slidet::app::Mode::Browse,
        scroll: 0,
        should_quit: false,
        image: ImageContext {
            image_picker: None,
            image_states: std::collections::HashMap::new(),
        },
    };

    // 3. 主事件循环
    loop {
        terminal.draw(|frame| {
            let model = slidet::ui::RenderModel {
                slides: &app.slides,
                selected: app.selected,
                mode: slidet::ui::RenderMode::Browse,
                scroll: app.scroll,
            };
            render(frame, &model, &mut app.image)
        })?;

        // 处理输入事件...
        // if app.should_quit { break; }
    }

    // 4. 恢复终端状态
    restore_terminal()?;
    Ok(())
}
```

### 调试渲染输出

```rust
use slidet::ui::render_slide_content;

let markdown = "# Title\n\n- Item 1\n- Item 2\n\n```rust\nfn main() {}\n```";
let output = render_slide_content(None, markdown);
println!("{}", output);
// Output:
// Title (centered, yellow, bold+underline)
//
// - Item 1
// - Item 2
//
// [code:rust]
//     fn main() {}
```

<!-- END:USAGE_EXAMPLE -->
