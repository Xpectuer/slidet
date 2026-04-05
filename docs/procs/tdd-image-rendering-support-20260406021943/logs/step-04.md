## Step 4 — 更新回归样例以覆盖 PNG、SVG 与缺失资源

### Actions Taken
- 更新 `examples/04-markdown-regression/03-image-and-fallback.md`，同时覆盖可渲染 png、已存在 svg fallback 和缺失资源 fallback。
- 复用现有示例资源，未新增二进制 fixture。
- 对代码执行 `cargo fmt` 并完成全量测试。

### Verify Result
- `rg -n "terminal-flow.png|render-path.svg|not-found.png" examples/04-markdown-regression/03-image-and-fallback.md`
- `cargo test`
- 结果：通过
