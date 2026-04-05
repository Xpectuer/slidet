## Step 1 — 扩展图片准备与降级判定

### Actions Taken
- 在 `src/image.rs` 中加入 `svg` 扩展名检测，对已存在的 `svg` 返回明确的 fallback 文案。
- 保留缺失资源与终端能力不足时的 graceful fallback。
- 新增 `png` 可渲染、`svg` fallback、缺失资源 fallback 三个单元测试。

### Verify Result
- `cargo test image::tests`
- 结果：通过
