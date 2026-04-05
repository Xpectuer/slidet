---
title: "Reference: 图片渲染终端兼容性"
doc_type: reference
brief: "记录 slidet 当前图片渲染链路、终端能力判定与已知兼容性注意事项"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# 图片渲染终端兼容性

## 当前行为

- `png`、`jpg`、`jpeg` 资源在被识别为“终端支持图片”时，会进入真实图片渲染链路。
- `svg` 当前不做真渲染，统一显示明确的文本 fallback。
- 缺失资源或终端能力不足时，统一显示可读提示，不应 panic。

## 代码边界

- `src/image.rs` 负责图片资源分类与基础能力判定。
- `src/app.rs` 持有 `ratatui-image` 的 `Picker` 和按路径缓存的 `StatefulProtocol`。
- `src/ui.rs` 按 Markdown block 顺序渲染文本和图片，两种模式共用同一链路。

## 已验证终端判定

- `KITTY_WINDOW_ID` 存在时，视为支持图片。
- `TERM_PROGRAM=iTerm.app` 时，视为支持图片。
- `TERM_PROGRAM=WezTerm` 时，视为支持图片。
- `TERM_PROGRAM=ghostty` 时，视为支持图片。

## 已知注意事项

- 仅依赖环境变量做“是否支持图片”的快速判定，可能遗漏其他支持图像协议的终端。
- `App::default_image_picker()` 仍会在支持图片的终端内调用 `Picker::from_query_stdio()` 获取更具体的协议与字体信息；如果查询失败，会退回 `Picker::from_fontsize((10, 20))`。
- 当用户反馈“示例中 png 没有显示”时，优先检查当前终端的 `TERM_PROGRAM`/`KITTY_WINDOW_ID`，再看是否是资源路径或 `Picker` 查询问题。

## 本次会话结论

- `examples/02-image-demo` 中的 `terminal-flow.png` 资源本身正常。
- 该示例在 Ghostty 中未显示 png 的直接原因，是终端能力白名单漏掉了 `ghostty`。
- 修正后，Ghostty 会进入真实图片渲染链路。
