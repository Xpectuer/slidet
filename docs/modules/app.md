---
doc_type: module
module_name: app
module_path: src/app.rs
generated_by: mci-phase-2
created: 2026-04-06
revision: 2
brief: 应用状态、事件循环、按键处理、图片状态缓存
---

# App Module

核心应用模块，负责管理幻灯片播放器的全局状态、处理用户输入事件、维护图片渲染缓存，以及驱动主事件循环。

<!-- BEGIN:INTERFACE -->
## Interface

### 公开类型

| 类型 | 描述 |
|------|------|
| `Mode` | 枚举类型，表示当前显示模式：`Browse`（浏览）或 `Present`（演示） |
| `App` | 主应用状态结构体，包含所有运行时状态 |
| `ImageContext` | 图片渲染上下文，封装图片选择器和状态缓存 |

### 公开函数

| 函数签名 | 描述 |
|----------|------|
| `App::new(slides: Vec<Slide>) -> Self` | 使用默认图片选择器创建 App 实例 |
| `App::current_slide(&self) -> &Slide` | 获取当前选中幻灯片的不可变引用 |
| `App::next_slide(&mut self)` | 切换到下一张幻灯片（如果存在） |
| `App::previous_slide(&mut self)` | 切换到上一张幻灯片（如果存在） |
| `App::handle_key(&mut self, code: KeyCode)` | 处理键盘事件，更新应用状态 |
| `App::image_state_for(&mut self, path: &Path) -> Result<&mut StatefulProtocol>` | 代理到图片上下文，获取或创建图片渲染状态 |
| `ImageContext::image_state_for(&mut self, path: &Path) -> Result<&mut StatefulProtocol>` | 获取或创建图片的渲染状态（带缓存） |
| `run(terminal: &mut DefaultTerminal, slides: Vec<Slide>) -> Result<()>` | 主事件循环入口函数 |
<!-- END:INTERFACE -->

<!-- BEGIN:DEPENDENCIES -->
## Dependency Graph

### 内部模块依赖

```
app
├── crate::loader::Slide    -- 幻灯片数据结构
├── crate::ui::{render, RenderModel, RenderMode, ImageStateStore}
│                           -- UI 渲染函数和渲染模型接口
└── crate::image::terminal_supports_images  -- 终端图片能力探测
```

### 外部依赖

| Crate | 用途 |
|-------|------|
| `anyhow` | 错误处理（`Result`, `Context`） |
| `crossterm` | 终端事件处理（`Event`, `KeyCode`, `KeyEventKind`） |
| `ratatui` | 终端 UI 框架（`DefaultTerminal`） |
| `ratatui_image` | 图片渲染（`Picker`, `StatefulProtocol`） |
| `image` | 图片加载（`DynamicImage::open`） |
<!-- END:DEPENDENCIES -->

<!-- BEGIN:STATE_MANAGEMENT -->
## State Management

### App 结构体

```rust
pub struct App {
    pub slides: Vec<Slide>,                    // 幻灯片列表
    pub selected: usize,                       // 当前选中索引
    pub mode: Mode,                            // 显示模式
    pub scroll: u16,                           // 垂直滚动偏移
    pub should_quit: bool,                     // 退出标志
    pub image: ImageContext,                   // 图片渲染上下文
}

pub struct ImageContext {
    pub image_picker: Option<Picker>,          // 图片选择器（可选）
    pub image_states: HashMap<PathBuf, StatefulProtocol>,  // 图片状态缓存
}
```

### 状态管理策略

1. **幻灯片导航状态**：`selected` 索引 + `scroll` 偏移组合控制当前视图
2. **模式切换**：`Mode` 枚举控制 Browse/Present 两种显示模式
3. **图片状态缓存**：`ImageContext` 使用 `HashMap<PathBuf, StatefulProtocol>` 按路径缓存图片渲染状态，避免重复解码和协议初始化
4. **渲染解耦**：`run()` 每帧构造 `ui::RenderModel`，把只读 UI 状态和可变图片缓存分开传给 `ui`

### 状态转换规则

| 触发条件 | 状态变更 |
|----------|----------|
| `next_slide()` | `selected += 1`, `scroll = 0` |
| `previous_slide()` | `selected -= 1`, `scroll = 0` |
| `Enter` 键 | `mode = Present`, `scroll = 0` |
| `Esc` 键 | `mode = Browse`, `scroll = 0` |
| `PageDown` | `scroll += 5` |
| `PageUp` | `scroll -= 5` |
<!-- END:STATE_MANAGEMENT -->

<!-- BEGIN:EDGE_CASES -->
## Edge Cases

### 硬编码值

| 值 | 位置 | 说明 |
|----|------|------|
| `5` | `handle_key()` PageDown/PageUp | 每次滚动的行数 |
| `(10, 20)` | `default_image_picker()` | fallback 字体尺寸 |
| `(8, 16)` | tests | 测试用字体尺寸 |

### 按键映射

| 按键 | 行为 |
|------|------|
| `j` / `Down` | 下一张幻灯片 |
| `k` / `Up` | 上一张幻灯片 |
| `Enter` | 进入演示模式 |
| `Esc` | 返回浏览模式 |
| `PageDown` | 向下滚动 5 行 |
| `PageUp` | 向上滚动 5 行 |
| `q` | 退出程序 |

### 边界检查

1. **幻灯片导航**：`next_slide()` 检查 `selected + 1 < slides.len()`，`previous_slide()` 检查 `selected > 0`
2. **滚动溢出**：使用 `saturating_add` / `saturating_sub` 防止 u16 溢出
3. **图片能力降级**：若终端不支持图片，`image.image_picker` 为 `None`，`image_state_for()` 返回错误

### 错误处理

| 场景 | 错误类型 | 消息 |
|------|----------|------|
| 终端不支持图片 | `anyhow::Error` | "image rendering is unavailable for this terminal" |
| 图片解码失败 | `anyhow::Error` | "failed to decode image {path}" |
| 缓存未命中（逻辑错误） | `anyhow::Error` | "missing cached image state after initialization" |
<!-- END:EDGE_CASES -->

<!-- BEGIN:USAGE_EXAMPLE -->
## Usage Example

```rust
use slidet::app::{run, App, Mode};
use slidet::loader::Slide;
use ratatui::DefaultTerminal;

// 1. 加载幻灯片
let slides = vec![
    Slide { path: "01.md".into(), title: "Intro".into(), raw_markdown: "...".into() },
    Slide { path: "02.md".into(), title: "Content".into(), raw_markdown: "...".into() },
];

// 2. 创建终端
let mut terminal = ratatui::init();

// 3. 运行应用
match run(&mut terminal, slides) {
    Ok(()) => println!("正常退出"),
    Err(e) => eprintln!("错误: {}", e),
}

// 4. 恢复终端状态
ratatui::restore();

// 单元测试风格：直接操作 App 状态
let mut app = App::new(test_slides);
app.handle_key(KeyCode::Down);      // 移动到下一张
assert_eq!(app.selected, 1);
app.handle_key(KeyCode::Enter);     // 进入演示模式
assert_eq!(app.mode, Mode::Present);
app.handle_key(KeyCode::Char('q')); // 请求退出
assert!(app.should_quit);
```
<!-- END:USAGE_EXAMPLE -->
