---
title: "docs/ Index"
doc_type: reference
brief: "Top-level index of all documentation in docs/"
confidence: verified
created: 2026-05-23
updated: 2026-05-23
revision: 1
---

# docs/ Index

Progressive-discovery index for agent documentation. Load the doc whose trigger condition matches your task.

## Start Here (first session, onboarding)

| Document | Load when | Brief | Size |
|----------|-----------|-------|------|
| [ARCHITECTURE.md](../ARCHITECTURE.md) | First session, understanding the system | System overview, components, design decisions, key paths | ~3K tokens |

## Reference Indices

| Document | Load when | Brief |
|----------|-----------|-------|
| [modules/index.md](modules/index.md) | Exploring module boundaries, dependency graphs, or public APIs | Auto-generated index of all module docs with dependency graph and global interface index |
| [lessons/index.md](lessons/index.md) | Searching for prior lessons or looking for patterns from past work | Auto-generated index of lesson documents |

## Working with the Codebase

| Document | Load when | Brief | Size |
|----------|-----------|-------|------|
| [modules/markdown.md](modules/markdown.md) | Modifying markdown parsing, adding block/span types, or collecting links | Markdown parser: block model, inline spans, link collection API | ~2K tokens |
| [modules/ui.md](modules/ui.md) | Modifying rendering, adding new visual elements, or debugging layout | UI renderer: Browse/Present views, inline span rendering, hardcoded colors | ~2K tokens |
| [modules/app.md](modules/app.md) | Modifying keybindings, event loop, or application state management | App state, key mappings, event loop, image caching, link opening | ~2K tokens |
| [modules/loader.md](modules/loader.md) | Changing slide loading, directory scanning, or file ordering | Slide loader: directory scan, .md file loading, filename ordering | ~1.5K tokens |
| [modules/image.md](modules/image.md) | Debugging image rendering, adding terminal support, or modifying fallback logic | Image renderer: terminal detection, graceful degradation, SVG handling | ~1K tokens |
| [modules/watcher.md](modules/watcher.md) | Debugging file watching, hot reload issues, or debouncing config | File watcher: filesystem monitoring, .md change detection, hot reload | ~1K tokens |
| [lessons/ratatui-link-rendering.md](lessons/ratatui-link-rendering.md) | Adding link features, using OSC 8 escape sequences, or terminal URL handling | Why OSC 8 breaks ratatui layout, and the gray-URL + terminal-detection approach | ~1.5K tokens |
| [lessons/unified-markdown-pipeline.md](lessons/unified-markdown-pipeline.md) | Evaluating markdown library choices or considering pipeline refactoring | Migration from split rendering (two libraries) to unified pulldown-cmark | ~1.5K tokens |
| [references/standard_markdown_ref.md](references/standard_markdown_ref.md) | Implementing new markdown features or testing edge cases | Reference document for standard markdown syntax and edge cases | ~500 tokens |
| [references/image-rendering-terminal-compat.md](references/image-rendering-terminal-compat.md) | Debugging image display issues or adding terminal support | Terminal image rendering compatibility reference | ~500 tokens |
| [quality/codebase-review.md](quality/codebase-review.md) | Understanding code quality status or planning refactoring | Codebase quality review and audit findings | ~500 tokens |

## Design Artifacts (drafts)

| Document | Load when | Brief |
|----------|-----------|-------|
| [drafts/](drafts/) | Reviewing past design sessions or understanding feature rationale | Intake sessions, specs, plans, and reviews for major features |

## Execution Artifacts (procs)

| Document | Load when | Brief |
|----------|-----------|-------|
| [procs/](procs/) | Tracing TDD execution history, reviewing step logs, or debugging past implementations | TDD execution logs, step-by-step implementation records |

## Frontmatter Status Reference

| Status | Meaning |
|--------|---------|
| `activated` | Activated from intake, execution is in progress |
| `ready` | Design phase complete, ready for activation |
| `completed` | Execution is complete |
