---
title: "Spec: Slide t 终端 Markdown Slide 播放器"
doc_type: proc
brief: "Rust ratatui 终端 slide 播放器，左侧 Markdown slide 列表，右侧渲染与全屏播放"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Spec: Slide t 终端 Markdown Slide 播放器

## Solution Summary

在 `slidet/` 下实现一个新的 Rust 终端应用 `Slide t`，用 `ratatui` 提供 Markdown 课件的浏览与播放体验。程序从命令行接收一个 slide 目录，把目录中的每个 `.md` 文件视为一张 slide，并按文件名字典序构建左侧列表与右侧当前页渲染。第一版只覆盖 Markdown 文本、图片、基础键盘导航和全屏播放切换，不引入 manifest、自动播放、speaker notes 或复杂演示特性。

## Acceptance Criteria

- [ ] `slidet/` 下存在可构建运行的 Rust `ratatui` 项目骨架
- [ ] 程序可接收一个目录作为 slide 数据源，并识别其中的 `.md` 文件
- [ ] 左栏能列出该目录中的 Markdown slides，顺序基于文件名
- [ ] 右栏能渲染当前选中的 Markdown slide 文本内容
- [ ] 用户可通过基础键盘操作切换当前 slide
- [ ] 用户可进入播放模式，并让当前 slide 内容铺满 terminal
- [ ] Markdown 中的图片在支持的 terminal 中尝试真实渲染
- [ ] 当 terminal 不支持图片协议时，程序不会崩溃，并提供可理解的降级显示
- [ ] 第一版实现不依赖 manifest、自动播放、speaker notes 等扩展特性
