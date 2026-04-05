---
title: "Spec: Markdown 幻灯片图片渲染"
doc_type: proc
brief: "为 slidet 首版补齐 jpg/png 真渲染，并为 svg 与异常场景定义稳定 fallback"
confidence: medium
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Spec

## Solution Summary
在现有 `markdown.rs` 图片块模型和 `image.rs` 能力探测基础上，为 `jpg`、`jpeg`、`png` 打通真实图片渲染链路，并在 `Browse` 与 `Present` 两种模式中统一采用“按可用区域等比缩放、完整显示优先”的策略。图片与文本继续按 Markdown 块顺序纵向展示，不引入复杂混排。`svg`、缺失文件和终端不支持图片三类场景继续走 graceful fallback，且文案保持简洁可读。

## Decisions
- 渲染范围限定为 `jpg`、`jpeg`、`png`；`svg` 首版不做真渲染，只显示明确占位文本，包含资源路径和“svg 暂不支持渲染”含义。
- `Browse` 与 `Present` 使用同一套图片加载与降级规则，不再输出 `[image render] <path>` 这类伪渲染占位。
- 图片显示策略为按当前可用渲染区域等比缩放，优先完整显示内容，不做裁剪优先策略。
- 当 slide 同时包含文本与图片时，保持 Markdown 原始块顺序，文本块与图片块纵向堆叠展示，不重排语义顺序。
- 终端不支持图片时，fallback 文案保持简洁，包含文件路径，不额外附带操作建议或终端名单。
- 缺失文件继续显示缺失提示；图片格式不支持或渲染失败时必须返回可读文本，不得 panic。
- 实现边界保持在现有模块内：`markdown.rs` 继续负责块解析，`image.rs` 负责资源解析与渲染准备，`ui.rs` 负责根据块类型渲染文本或图片。
- 自动化测试至少覆盖三类行为：可渲染图片链路、`svg` fallback、缺失文件 fallback；如现有示例不足，补充可回归的测试资源或样例目录。

## Open Questions
- 是否需要为图片渲染成功路径增加可提交的测试图片 fixture，还是通过更小粒度的单元测试隔离 `ratatui-image` 依赖。
- 首版在窄终端下的图片最小可视高度是否需要显式规则；当前默认按可用区域自然收缩处理。

## Acceptance Criteria
- [ ] Markdown 中的 `![alt](file.jpg)`、`![alt](file.jpeg)`、`![alt](file.png)` 在支持图片显示的终端中可实际显示为图片内容。
- [ ] `Browse` 模式与 `Present` 模式都使用相同的图片加载、缩放和 fallback 规则。
- [ ] `svg` 资源显示明确文本占位，用户可分辨“文件存在但格式暂不支持渲染”。
- [ ] 图片文件缺失时继续显示缺失提示；终端不支持图片时继续显示简洁能力不足提示；两种情况都不得 panic。
- [ ] 图文混合 slide 中，文本与图片按 Markdown 原始顺序纵向展示，不发生隐式重排。
- [ ] 自动化测试覆盖至少三类情况：`jpg/png` 可渲染链路、`svg` fallback、缺失文件 fallback。
