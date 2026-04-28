# Project Guide

This file provides guidance to AI coding assistants (Claude Code, Cursor, Windsurf, etc.)
when working with code in this repository.

## ⚠️ 重要：架构文档阅读要求

**在进行任何 feature 修改或 debug 前，必须先阅读 [ARCHITECTURE.md](ARCHITECTURE.md)**

这确保你了解：
- 系统整体架构和数据流
- 模块间的依赖关系
- 关键路径和配置驱动逻辑
- 已知限制和未来扩展点

## Project Overview

`slidet` 是一个用 Rust 编写的终端 Markdown 幻灯片播放器。当前实现使用 `clap` 解析命令行参数，使用 `ratatui`/`crossterm` 渲染交互式 TUI，使用 `pulldown-cmark` 将每个 `.md` 文件解析为可展示的文本与图片块。支持文件系统热重载：编辑 `.md` 文件后自动刷新幻灯片内容，右下角显示"Reloaded"指示器。

运行方式是把一个目录传给 `slidet`，程序会按文件名字典序加载其中的 `.md` 文件作为 slides：

```bash
cargo run -- examples/01-text-lecture
```

当前交互模型分为两种模式：

- `Browse`：左侧 slide 列表，右侧当前 slide 预览
- `Present`：全屏展示当前 slide

默认按键约定：

- `j` / `Down`：下一页
- `k` / `Up`：上一页
- `Enter`：进入全屏展示
- `Esc`：返回浏览模式
- `PageDown` / `PageUp`：滚动当前内容
- `q`：退出

## Repository Structure

```
.
├── Cargo.toml                # crate 清单；核心依赖为 clap / ratatui / pulldown-cmark / notify-debouncer-mini
├── src/
│   ├── main.rs               # CLI 入口，加载 slides，初始化/恢复终端
│   ├── lib.rs                # 导出各模块
│   ├── loader.rs             # 扫描目录并加载 .md slides
│   ├── markdown.rs           # 将 Markdown 解析为文本块和图片块
│   ├── image.rs              # 图片能力探测与降级策略
│   ├── app.rs                # 应用状态、按键处理、主事件循环（轮询模式 + 热重载）
│   ├── ui.rs                 # Browse/Present 两种视图渲染 + 重载指示器
│   └── watcher.rs            # 文件系统监控（notify-debouncer-mini），检测 .md 变更
├── examples/
│   ├── 01-text-lecture/      # 纯文本演示样例
│   ├── 02-image-demo/        # 图片与 fallback 演示
│   ├── 03-engineering-notes/ # 工程说明样例
│   ├── 04-markdown-regression/ # Markdown 回归样例
│   ├── 05-parser-edge-cases/ # 解析边界样例
│   └── 06-slide-navigation-story/ # 导航故事样例
├── docs/
│   ├── drafts/               # 需求、规划、验证草稿
│   ├── procs/                # 执行日志、TDD 过程文档
│   ├── issues/               # issue 模板和问题跟踪
│   ├── rules/                # 项目规则
│   ├── modules/              # 模块级文档
│   └── sops/                 # SOP 模板和沉淀文档
└── scripts/
    ├── fm.sh                 # frontmatter 字段读取
    ├── rebuild-indexes.sh    # 重建 docs 索引
    ├── init-proc.sh          # 初始化执行过程目录
    ├── next-steps.sh         # 汇总后续步骤
    └── test-*.sh             # 文档/扫描相关脚本
```

## Development Guidelines

- 优先把行为放在 `src/` 中已有模块边界内实现，不要把 loader、parser、UI、状态管理、watcher 逻辑混在一起。
- Slide 的来源约定是”一个目录下的多个 `.md` 文件”，并且依赖文件名字典序控制播放顺序；改动加载逻辑时不要破坏这个约定。
- Markdown 当前被解析为 `Text` 和 `Image` 两类块。扩展语法时，先确认 `markdown.rs` 的块模型是否需要演进，再修改 `ui.rs` 的渲染逻辑。
- 图片能力必须保留 graceful fallback。即使终端不支持图片、资源缺失或加载失败，程序也应输出可读占位文本，而不是 panic。
- 终端生命周期要成对处理：初始化后必须恢复终端状态。修改 `main.rs`、`ui.rs` 或事件循环时，不要引入”异常退出后终端未恢复”的回归。
- 事件循环使用轮询模式（`event::poll(100ms)`），不是阻塞 `event::read()`。这允许每帧非阻塞检查文件变更。改动事件循环时保持这个轮询结构。
- 文件监控是可选的（`Option<SlideWatcher>`）。即使文件系统监控不可用，程序也应正常运行（只是没有热重载）。
- 已有单元测试覆盖 `loader`、`markdown`、`image`、`app`、`ui`、`watcher` 的基础行为；修改这些模块时，优先补测试再改实现。
- 示例目录不仅是 demo，也承担回归样本的作用。改解析或渲染行为时，优先复用或补充 `examples/04-*`、`examples/05-*` 中的样例。

## Git History Notes

- 项目已有 10+ 个提交，涵盖初始骨架、Markdown 管线统一、图片渲染支持、CI/CD 发布自动化等。
- 早期提交标题 `850f2a6 Initial commit: Rust project skeleton` 虽然写的是 skeleton，但实际已包含完整的最小可用实现。
- 最近的主要功能：文件监控热重载（`watcher.rs` + 事件循环轮询重构）尚未提交。

## Common Commands

```bash
# 运行一个示例 slide 目录
cargo run -- examples/01-text-lecture

# 执行测试
cargo test

# 快速检查编译
cargo check

# 重建 docs 索引
scripts/rebuild-indexes.sh --project-dir .
```

## Rules

No rules defined yet. Add rules to `docs/rules/` and run `scripts/rebuild-indexes.sh --project-dir .`

## Docs Guide

| Directory | Purpose | When to Use |
|-----------|---------|-------------|
| `docs/rules/` | Coding rules and standards | Before writing code |
| `docs/sops/` | Standard operating procedures | Repeatable workflows |
| `docs/drafts/` | Design-phase artifacts | intake→idea→plan |
| `docs/procs/` | Execution-phase tracking | tdd/progress/verify |
| `docs/issues/` | Issue tracking | Bug reports, investigations |

## Scripts

| Script | Usage |
|--------|-------|
| `scripts/fm.sh` | `fm.sh get <file> <field>` — frontmatter extraction |
| `scripts/rebuild-indexes.sh` | `rebuild-indexes.sh --project-dir .` — regenerate all index.md |

## 模块文档

详细模块文档位于 `docs/modules/`:

- **[模块文档索引](docs/modules/index.md)** - 所有模块的总索引、依赖关系图和全局接口索引
- [loader 模块](docs/modules/loader.md) - 目录扫描和幻灯片加载
- [markdown 模块](docs/modules/markdown.md) - Markdown 解析和结构化块模型
- [image 模块](docs/modules/image.md) - 终端图片能力检测和降级策略
- [app 模块](docs/modules/app.md) - 应用状态、事件循环和热重载
- [ui 模块](docs/modules/ui.md) - Browse/Present 双模式渲染和重载指示器
- [watcher 模块](docs/modules/watcher.md) - 文件系统监控和热重载触发

每个模块文档包含：
- Interface：公开的函数、类型和 API
- Dependency Graph：依赖的内部模块
- State Management：状态管理方式
- Edge Cases：硬编码值和错误处理
- Usage Example：使用示例
