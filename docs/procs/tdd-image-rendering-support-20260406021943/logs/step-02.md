## Step 2 — 在线程安全边界内挂接图片渲染状态

### Actions Taken
- 在 `src/app.rs` 中为 `App` 增加 `image_picker` 和 `image_states` 缓存。
- 增加 `image_state_for`，按路径懒加载并复用 `StatefulProtocol`。
- 将 `run` 中的 UI 绘制入口切换为 `&mut App`。

### Verify Result
- `cargo test app::tests`
- 结果：通过
