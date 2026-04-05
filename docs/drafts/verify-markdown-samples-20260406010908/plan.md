---
title: "Plan: slidet Markdown 示例输入验证"
doc_type: proc
brief: "为 slidet 的 Markdown 示例输入设计生成、检查和回归步骤"
confidence: speculative
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Plan: slidet Markdown 示例输入验证

## Inputs

- `docs/drafts/verify-markdown-samples-20260406010908/requirements.md`
- `docs/drafts/verify-markdown-samples-20260406010908/spec.md`
- `docs/procs/tdd-slidet-terminal-markdown-slide-player-20260406003915/tdd.md`

## Step 1 — Expand Example Corpus

**What**: 在 `slidet/examples/` 下新增更系统的 Markdown 样例目录，覆盖复杂文本结构、parser 边界和多页演示故事线。

**Verify**:

```bash
find slidet/examples -maxdepth 2 -type f -name '*.md' | sort
```

## Step 2 — Check Directory Ordering Assumptions

**What**: 确认每个样例目录的 slide 文件采用前缀编号命名，能被 `loader` 稳定按字典序读取。

**Verify**:

```bash
cargo test --manifest-path slidet/Cargo.toml loader
```

## Step 3 — Exercise Markdown Parsing Scenarios

**What**: 用新增样例覆盖标题、段落、分隔线、引用、列表、代码块、图片和缺图引用，作为 parser 回归输入。

**Verify**:

```bash
cargo test --manifest-path slidet/Cargo.toml markdown ui
```

## Step 4 — Run Manual Smoke Paths

**What**: 对至少两组样例执行本地浏览，验证 browse/present 模式、翻页和长内容滚动。

**Verify**:

```bash
cargo run --manifest-path slidet/Cargo.toml -- slidet/examples/04-markdown-regression
cargo run --manifest-path slidet/Cargo.toml -- slidet/examples/05-parser-edge-cases
```

## Step 5 — Close Verification

**What**: 记录哪些样例成为今后新增测试的 fixture 基线，并确认本轮 out-of-scope 没有被误扩展到 CI 或 golden testing。

**Verify**:

```bash
test -f docs/drafts/verify-markdown-samples-20260406010908/spec.md
test -f docs/drafts/verify-markdown-samples-20260406010908/session-log.md
```
