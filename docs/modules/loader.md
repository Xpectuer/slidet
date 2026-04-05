---
title: "Loader Module"
doc_type: module
module_name: loader
module_path: src/loader.rs
generated_by: mci-phase-2
brief: "Directory scanning and Markdown slide loading with lexicographic ordering"
confidence: verified
created: 2026-04-06
updated: 2026-04-06
revision: 1
---

# Loader Module

## Overview

The loader module is responsible for scanning a directory, filtering Markdown files, sorting them by filename, and loading their contents into `Slide` structs. It provides the entry point for slide discovery and content acquisition in the slidet application.

<!-- BEGIN:Interface -->
## Interface

### Public Types

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slide {
    pub path: PathBuf,        // Absolute or relative path to the .md file
    pub title: String,        // Derived from filename stem (without extension)
    pub raw_markdown: String, // Full file contents as UTF-8 string
}
```

### Public Functions

```rust
/// Load all Markdown slides from a directory.
///
/// # Arguments
/// * `dir` - Path to the directory containing .md files
///
/// # Returns
/// * `Ok(Vec<Slide>)` - Slides sorted by filename (lexicographic order)
/// * `Err(anyhow::Error)` - If directory is invalid, unreadable, or contains no .md files
///
/// # Errors
/// - Directory does not exist
/// - Path is not a directory
/// - Directory contains no .md files
/// - File read failure
pub fn load_slides(dir: &Path) -> Result<Vec<Slide>>
```
<!-- END:Interface -->

<!-- BEGIN:DependencyGraph -->
## Dependency Graph

```
loader.rs
    |
    +-- std::fs (file system operations)
    |       |-- read_dir()
    |       |-- read_to_string()
    |
    +-- std::path::{Path, PathBuf}
    |       |-- extension()
    |       |-- file_stem()
    |       |-- is_dir()
    |       |-- exists()
    |
    +-- anyhow::{bail, Context, Result}
            |-- Error propagation with context
```

### Internal Module Dependencies

The loader module has **no dependencies** on other internal modules (`app`, `image`, `markdown`, `ui`). It produces `Slide` structs that are consumed by:

| Consumer | Usage |
|----------|-------|
| `src/main.rs` | Calls `load_slides()` to initialize the application state |
| `src/markdown.rs` | Receives `Slide::raw_markdown` for parsing into blocks |
| `src/ui.rs` | Uses `Slide::title` for slide list display |
<!-- END:DependencyGraph -->

<!-- BEGIN:StateManager -->
## State Management

The loader module is **stateless**. All operations are performed through pure functions:

1. **No mutable global state**: `load_slides()` takes an input and returns a result without side effects.
2. **Immutable output**: The returned `Vec<Slide>` is owned by the caller; `Slide` fields are all immutable references in practice.
3. **Deterministic ordering**: Given the same directory contents, `load_slides()` always returns slides in the same lexicographic order.

### Memory Model

```
┌─────────────────────────────────────────────────────────┐
│  load_slides(dir)                                       │
│      │                                                  │
│      ▼                                                  │
│  ┌─────────────┐    ┌─────────────────────────────────┐│
│  │ PathBuf     │───▶│ Vec<Slide> (owned)              ││
│  │ (borrowed)  │    │  - path: PathBuf                ││
│  └─────────────┘    │  - title: String                ││
│                     │  - raw_markdown: String         ││
│                     └─────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

The entire file contents are loaded into memory. For large slides or many files, consider memory implications.
<!-- END:StateManager -->

<!-- BEGIN:EdgeCases -->
## Edge Cases

### Hardcoded Values

| Value | Location | Purpose |
|-------|----------|---------|
| `"md"` | Line 28 | File extension filter for Markdown files |
| `"untitled"` | Line 43 | Default title when filename stem extraction fails |

### Error Handling

| Condition | Error Message Pattern | Recovery |
|-----------|----------------------|----------|
| Directory does not exist | `"slides directory does not exist: {path}"` | None - fatal error |
| Path is not a directory | `"slides path is not a directory: {path}"` | None - fatal error |
| Directory read failure | `"failed to read directory {path}"` | None - fatal error |
| Entry enumeration failure | `"failed to enumerate {path}"` | None - fatal error |
| No .md files found | `"no markdown slides found in {path}"` | None - fatal error |
| File read failure | `"failed to read {path}"` | None - fatal error |

### Behavior Notes

1. **Hidden files**: Files starting with `.` are included if they have `.md` extension. Consider filtering if this is undesirable.
2. **Symlinks**: Symlinks to files are followed; symlinks to directories within the target are not recursively scanned.
3. **Unicode filenames**: Handled correctly via `PathBuf` and `to_string_lossy()` for title extraction.
4. **Empty files**: Loaded successfully; `raw_markdown` will be an empty string.
5. **Binary files with .md extension**: Will cause UTF-8 decode error on `read_to_string()`.
<!-- END:EdgeCases -->

<!-- BEGIN:UsageExample -->
## Usage Example

```rust
use slidet::loader::{load_slides, Slide};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Define the slides directory
    let slides_dir = Path::new("examples/01-text-lecture");

    // Load all slides with error handling
    let slides = load_slides(slides_dir)?;

    // Iterate over slides in sorted order
    for (index, slide) in slides.iter().enumerate() {
        println!(
            "Slide {}: {} ({} bytes)",
            index + 1,
            slide.title,
            slide.raw_markdown.len()
        );
    }

    // Access specific slide content
    if let Some(first) = slides.first() {
        println!("First slide path: {:?}", first.path);
    }

    Ok(())
}
```

### Expected Output (for `examples/01-text-lecture/`)

```
Slide 1: 01-intro (156 bytes)
Slide 2: 02-content (892 bytes)
Slide 3: 03-conclusion (234 bytes)
First slide path: "examples/01-text-lecture/01-intro.md"
```
<!-- END:UsageExample -->
