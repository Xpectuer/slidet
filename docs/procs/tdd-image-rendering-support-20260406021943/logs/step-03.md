## Step 3 — 用真实图片 widget 替换字符串占位

### Actions Taken
- 在 `src/ui.rs` 中将 `render`、`render_browse`、`render_present` 改为接受 `&mut App`。
- 增加按 Markdown block 顺序渲染文本和图片的共用逻辑。
- 对可渲染图片使用 `StatefulImage::default().resize(Resize::Fit(None))`，失败场景保留文本 fallback。
- 增补 UI 测试，确认缺失资源提示仍可见，且不再输出 `[image render]` 字符串占位。

### Verify Result
- `cargo test ui::tests`
- 结果：通过
