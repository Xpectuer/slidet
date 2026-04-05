| Step | Status | Notes |
|------|--------|-------|
| Step 1 — 扩展图片准备与降级判定 | ✅ | `src/image.rs` 增加 svg fallback，并补齐缺失资源、svg、png 三类测试 |
| Step 2 — 在线程安全边界内挂接图片渲染状态 | ✅ | `App` 持有 `Picker` 与图片状态缓存，渲染入口改为可变借用 |
| Step 3 — 用真实图片 widget 替换字符串占位 | ✅ | `Browse`/`Present` 共用块级渲染链路，终端支持时走 `StatefulImage` |
| Step 4 — 更新回归样例以覆盖 PNG、SVG 与缺失资源 | ✅ | 回归样例同时覆盖 png、svg fallback 与缺失资源，并通过全量测试 |
| Follow-up — Ghostty 终端识别 | ✅ | 将 `ghostty` 纳入图片终端支持判定，修复 `examples/02-image-demo` 中 png 被误降级的问题 |
