---
title: "Spec: Markdown 幻灯片图片渲染"
doc_type: proc
brief: "为 slidet 首版补齐 jpg/png 真渲染，并让 svg 明确降级显示"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Spec: Markdown 幻灯片图片渲染

## Solution Summary
在现有 `markdown.rs` 图片块模型和 `image.rs` 能力探测基础上，首版实现将把 `jpg` 与 `png` 图片从“字符串占位”升级为真实终端图片渲染，并在 `Browse` 与 `Present` 两种模式下统一采用同一套加载、缩放和降级规则。`svg`、缺失资源和终端能力不足三类场景继续走 `graceful fallback`，但文案会更明确，让用户能区分“资源不存在”“终端不支持图片”和“格式暂不支持渲染”。

## Acceptance Criteria

- [ ] Markdown 中的 `![alt](file.jpg)` 与 `![alt](file.png)` 在支持图片显示的终端中可实际显示为图片内容。
- [ ] `Browse` 模式与 `Present` 模式都遵循相同的图片加载与降级规则，不再只输出 `[image render] <path>`。
- [ ] `svg` 资源在首版显示为明确文本占位，能让用户分辨“文件存在但格式暂不支持渲染”。
- [ ] 图片文件缺失时继续显示缺失提示；终端不支持图片时继续显示能力不足提示；两种情况都不得 panic。
- [ ] 自动化测试覆盖至少三类情况：`jpg/png` 可渲染链路、`svg` fallback、缺失文件 fallback。
