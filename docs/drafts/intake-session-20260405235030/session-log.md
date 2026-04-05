---
title: "Intake Session Log"
doc_type: proc
status: activated
brief: "Rust ratatui 终端 slide 播放器，左侧 Markdown slide 列表，右侧渲染与全屏播放"
confidence: speculative
created: 2026-04-05
updated: 2026-04-06
revision: 2
context_ref: "docs/references/context-session-20260405235030/"
---

# Intake Session Log

**Session**: intake-session-20260405235030
**Status**: active | ready | activated | abandoned

## Context Snapshot
- 仓库当前以 Markdown / MkDocs / Python 工具为主，尚无 Rust TUI 子项目
- `scripts/fm.sh` 与 `scripts/next-steps.sh` 已补齐，可支持 intake 流程
- 现有 `docs/` 结构适合记录本次新项目需求
- 本次需求将引入新的 Rust 子目录 `slidet/`

## Q&A Transcript

### Round 1
**Category**: Goal
**Question**: 这次要继续旧会话，还是为 `Slide t` 新建 intake？
**Answer**: 新建本次 `Slide t` intake。
**Files Read**: `docs/drafts/intake-20260308120000/requirements.md`

### Round 2
**Category**: Goal
**Question**: `Slide t` 第一版的核心目标更偏向哪一种？
**Answer**: Markdown 课件播放器。
**Files Read**: None

### Round 3
**Category**: Existing State
**Question**: 第一版里，“左侧 slide 列表的一组 Markdown” 希望按哪种文件组织方式读取？
**Answer**: 一个目录下每个 `.md` 文件就是一张 slide。
**Files Read**: None

### Round 4
**Category**: Scope
**Question**: 第一版里，Markdown 里的图片希望怎么渲染到 terminal？
**Answer**: 支持终端图片协议，有能力则显示图片，否则显示占位或路径。
**Files Read**: None

### Round 5
**Category**: Acceptance
**Question**: 第一版播放和导航的交互，希望最小可用集合是哪一种？
**Answer**: 基础键盘导航加全屏播放切换。
**Files Read**: None

### Round 6
**Category**: Constraints
**Question**: 第一版的运行环境，希望把目标平台限定到什么程度？
**Answer**: 先以 macOS 常见 terminal 为主，其他环境尽量兼容。
**Files Read**: None

## Summary
**Rounds**: 6
**Stop Reason**: criteria met
**Gaps**:
- [UNCERTAIN] 具体键位设计尚未确定
- [UNCERTAIN] 图片协议优先级与支持矩阵尚未确定

## Final Synthesis

本次 intake 聚焦一个新的 Rust 子项目 `Slide t`。项目目录位于 `slidet/`，产品定位是终端内的 Markdown 课件播放器，而不是泛用文档查看器。第一版的数据源为文件系统目录，目录中的每个 `.md` 文件对应一张 slide，程序按文件名顺序组织 slide 列表。主界面采用左右两栏布局：左栏是 slide 文件列表，右栏渲染当前 slide。进入播放模式后，右栏内容需要铺满整个 terminal，以满足正式演示的场景。

第一版范围有意收窄，只支持 Markdown 文本与图片渲染。图片显示策略是优先使用终端图片协议做真实渲染；如果当前 terminal 不支持，则退化为占位或文件路径提示，保证可用性和稳定性。交互层面，第一版只保留最小闭环：基础键盘导航与进入/退出全屏播放，不引入自动播放、计时器、讲师视图、动画步进等扩展能力。目标运行环境优先是 macOS 常见 terminal，其他环境尽量兼容，但不以跨平台完全一致为第一阶段目标。
