---
title: "Spec: Unify Markdown Pipeline on pulldown-cmark"
doc_type: proc
brief: "Replace the split markdown rendering path with a single pulldown-cmark-driven pipeline"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Spec: Unify Markdown Pipeline on pulldown-cmark

## Solution Summary
`slidet` 将以 `pulldown-cmark` 作为唯一 Markdown 语义来源，在 `src/markdown.rs` 中把原始 Markdown 解析为适合终端展示的内部渲染模型，再由 `src/ui.rs` 仅负责把该模型渲染成 `ratatui` 文本和图片块。实现会移除 `tui-markdown` 驱动的文本渲染路径，同时保留现有 `Browse` / `Present` 交互、slide 加载顺序和图片 graceful fallback。第一版重点覆盖标题、列表、任务列表、引用、代码块、表格、图片和链接等常见元素，并在窄终端下优先保证稳定可读而不是严格还原原始排版。

## Acceptance Criteria
- [ ] `cargo check` 与 `cargo test` 通过。
- [ ] 示例 slides 中常见 Markdown 元素能够稳定显示。
- [ ] Markdown 显示行为围绕 `pulldown-cmark` 统一，而不是依赖分裂的多套 Markdown 语义路径。
