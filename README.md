# slidet

**Terminal Markdown Slide Player** — 在终端中播放你的 Markdown 幻灯片。

`slidet` 是一个用 Rust 编写的交互式终端 Markdown 幻灯片播放器。它将目录中的 `.md` 文件按文件名字典序组织成幻灯片，支持双模式浏览和全屏演示。

## 特性

- 📁 **目录即演示文稿** — 每一个 `.md` 文件都是一张幻灯片
- 🖼️ **图片支持** — 终端图片渲染，自动降级到占位文本
- 🎯 **双模式交互** — 浏览模式预览所有幻灯片，演示模式全屏展示
- ⌨️ **Vim 风格导航** — `j/k` 上下翻页，符合直觉的快捷键设计
- 🎨 **Markdown 富文本** — 基于 `pulldown-cmark` 的完整 Markdown 渲染

## 安装

### 从源码构建

```bash
git clone https://github.com/yourusername/slidet.git
cd slidet
cargo build --release
```

编译后的二进制文件位于 `target/release/slidet`。

## 快速开始

### 1. 准备幻灯片目录

创建一个目录，按播放顺序命名 Markdown 文件：

```
my-slides/
├── 01-introduction.md
├── 02-features.md
├── 03-demo.md
└── 04-conclusion.md
```

### 2. 运行 slidet

```bash
cargo run -- my-slides/
```

或使用编译后的版本：

```bash
./target/release/slidet my-slides/
```

### 3. 导航与交互

**浏览模式（默认）**
- `j` / `↓` — 下一张幻灯片
- `k` / `↑` — 上一张幻灯片
- `Enter` — 进入全屏演示模式
- `q` — 退出

**演示模式**
- `j` / `↓` / `PageDown` — 向下滚动内容
- `k` / `↑` / `PageUp` — 向上滚动内容
- `Esc` — 返回浏览模式
- `q` — 退出

## 示例

项目包含多个示例幻灯片集：

```bash
# 纯文本演示
cargo run -- examples/01-text-lecture

# 图片与降级策略演示
cargo run -- examples/02-image-demo

# 工程说明样例
cargo run -- examples/03-engineering-notes

# Markdown 回归测试
cargo run -- examples/04-markdown-regression

# 解析边界样例
cargo run -- examples/05-parser-edge-cases

# 导航故事样例
cargo run -- examples/06-slide-navigation-story
```

## Markdown 支持

`slidet` 使用 `pulldown-cmark` 解析器，支持标准 Markdown 语法：

```markdown
# 标题

**粗体** 和 *斜体*

- 列表项 1
- 列表项 2

`代码` 和代码块：

\`\`\`rust
fn main() {
    println!("Hello, slidet!");
}
\`\`\`

![图片描述](path/to/image.png)
```

### 图片支持

- 支持相对路径和绝对路径
- 自动检测终端图片能力
- 不支持时优雅降级为占位文本

## 架构

```
┌─────────────────────────────────────────────────────┐
│                        CLI                          │
│                   (clap + main.rs)                  │
└─────────────────────┬───────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│                    App State                        │
│              (app.rs + 事件循环)                     │
└─────┬───────────────────────────────┬───────────────┘
      │                               │
      ▼                               ▼
┌──────────────┐              ┌──────────────┐
│    Loader    │              │      UI      │
│ (loader.rs)  │              │   (ui.rs)    │
└──────┬───────┘              └──────┬───────┘
       │                             │
       ▼                             ▼
┌──────────────┐              ┌──────────────┐
│   Markdown   │              │   ratatui    │
│ (markdown.rs)│              │ + crossterm  │
└──────────────┘              └──────────────┘
```

详细架构文档见 [ARCHITECTURE.md](ARCHITECTURE.md)。

## 开发

### 依赖

- Rust 1.70+ (Edition 2021)
- 主要依赖：
  - `clap` — CLI 参数解析
  - `ratatui` — 终端 UI 框架
  - `crossterm` — 跨平台终端控制
  - `pulldown-cmark` — Markdown 解析
  - `ratatui-image` — 终端图片渲染

### 测试

```bash
# 运行所有测试
cargo test

# 快速检查编译
cargo check
```

### 文档

项目使用结构化文档系统：

- `docs/rules/` — 编码规则和标准
- `docs/sops/` — 标准操作流程
- `docs/modules/` — 模块详细文档
- `docs/drafts/` — 设计阶段文档
- `docs/procs/` — 执行跟踪

模块文档索引见 [docs/modules/index.md](docs/modules/index.md)。

## 贡献

欢迎贡献！请确保：

1. 阅读架构文档 [ARCHITECTURE.md](ARCHITECTURE.md)
2. 遵循现有代码风格和模块边界
3. 为新功能添加测试
4. 更新相关文档

## 许可证

MIT License

## 致谢

- [ratatui](https://github.com/ratatui-org/ratatui) — 优秀的终端 UI 框架
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) — 高性能 Markdown 解析器
- [clap](https://github.com/clap-rs/clap) — 强大的 CLI 框架
