---
title: "Verification Strategy: slidet Markdown 示例输入"
doc_type: proc
brief: "用结构化 Markdown 样例驱动 slidet 的解析与浏览回归"
confidence: speculative
created: 2026-04-06
updated: 2026-04-06
revision: 1
source_skill: verify
---

# Verification Strategy

## Scope

本轮验证聚焦 `slidet/examples/` 作为输入资产层的质量，目标是让 `slidet` 在面对更接近真实讲稿的 Markdown 目录时，能稳定完成加载、排序、文本解析、图片降级和浏览展示。

## Approach

采用“三层验证”：

1. 资产层：新增多组 Markdown 示例目录，显式覆盖文本、结构、图片和超长内容。
2. 自动化层：继续使用 `cargo test --manifest-path slidet/Cargo.toml`，后续可让 loader/parser/UI 单测直接消费这些样例目录。
3. 手工层：运行 `cargo run --manifest-path slidet/Cargo.toml -- <example-dir>`，在 browse/present 两种模式下做 smoke check。

## Test Matrix

| Component | Test Type | Framework | Priority | Status |
|-----------|-----------|-----------|----------|--------|
| `loader` 目录读取与排序 | 自动化 + 手工 smoke | Rust test + CLI run | P0 | planned |
| `markdown` 文本/图片块切分 | 自动化 + 样例回归 | Rust test | P0 | planned |
| `image` 缺图与可用图路径 | 自动化 + 手工 smoke | Rust test + CLI run | P0 | planned |
| `ui` 长文滚动与多页浏览 | 手工 smoke | CLI run | P1 | planned |
| 样例目录完整性 | 手工检查 | checklist | P1 | planned |

## Acceptance Criteria

1. `slidet/examples/` 中至少新增 3 组可排序、可直接运行的 Markdown 示例目录。
2. 新样例整体覆盖基础文本、结构化 Markdown、缺图回退、长文滚动、多页叙事五种场景中的至少四种。
3. 每个示例目录至少包含 2 个以上按序命名的 `.md` slide 文件。
4. 至少有一组样例包含存在的图片资源，至少一组样例包含故意缺失的图片资源。
5. verify 产物明确记录测试命令、手工 smoke 步骤和完成判定。

## Out of Scope

- 不引入截图 golden 测试
- 不改变 parser 支持范围
- 不建立 CI workflow
