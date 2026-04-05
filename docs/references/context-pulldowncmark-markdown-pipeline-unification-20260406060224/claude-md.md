# CLAUDE.md Snapshot

# Project Guide

This file provides guidance to AI coding assistants (Claude Code, Cursor, Windsurf, etc.)
when working with code in this repository.

## Project Overview

`slidet` 是一个用 Rust 编写的终端 Markdown 幻灯片播放器。当前实现使用 `clap` 解析命令行参数，使用 `ratatui`/`crossterm` 渲染交互式 TUI，使用 `pulldown-cmark` 将每个 `.md` 文件解析为可展示的文本与图片块。

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

```text
.
├── Cargo.toml
├── src/
├── examples/
├── docs/
└── scripts/
```

## Development Guidelines

- 优先把行为放在 `src/` 中已有模块边界内实现，不要把 loader、parser、UI、状态管理逻辑混在一起。
- Slide 来源约定是目录下多个 `.md` 文件，依赖文件名字典序控制顺序。
- Markdown 当前块模型是 `Text` 和 `Image`。
- 图片能力必须保留 graceful fallback。
- 终端生命周期初始化与恢复必须成对处理。
