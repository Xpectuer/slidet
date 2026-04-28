---
doc_type: module
module_name: watcher
module_path: src/watcher.rs
generated_by: manual
created: 2026-04-25
revision: 1
brief: File watching for hot-reload of slide content
---

# Watcher Module

Monitors the slide directory for `.md` file changes and signals the app to reload slides when changes are detected. Uses `notify-debouncer-mini` for cross-platform filesystem event debouncing.

<!-- BEGIN:INTERFACE -->
## Interface

### Public Types

| Type | Description |
|------|-------------|
| `SlideWatcher` | File watcher that detects `.md` file changes in a directory |

### Public Functions

| Function Signature | Description |
|---------------------|-------------|
| `SlideWatcher::new(dir: &Path) -> Result<Self>` | Creates a new watcher monitoring `dir` recursively. Returns error if filesystem watching is unavailable. |
| `SlideWatcher::poll_changes(&mut self) -> bool` | Non-blocking check for `.md` file changes since last call. Returns `true` if any qualifying event was found. |
<!-- END:INTERFACE -->

<!-- BEGIN:DEPENDENCIES -->
## Dependency Graph

### External Dependencies

| Crate | Usage |
|-------|-------|
| `notify-debouncer-mini` | Cross-platform filesystem event debouncing (200ms window) |
| `anyhow` | Error handling (`Result`) |

### Internal Dependencies

None. This is a leaf module with no dependencies on other slidet modules.
<!-- END:DEPENDENCIES -->

<!-- BEGIN:STATE_MANAGEMENT -->
## State Management

```rust
pub struct SlideWatcher {
    _debouncer: Debouncer,  // Held to keep the watcher alive (dropping stops watching)
    rx: Receiver<Result<Vec<DebouncedEvent>, notify::Error>>,  // Channel for debounced events
}
```

### Design Choices

1. **Debouncing**: 200ms debounce window prevents rapid reloads from editors that write multiple times per save.
2. **Channel-based**: Uses `std::sync::mpsc::channel` to decouple filesystem events from the polling loop.
3. **Non-blocking poll**: `poll_changes()` uses `try_iter()` to drain events without blocking the UI loop.
4. **`.md` filter**: Only events affecting files with `.md` extension trigger a reload signal.
<!-- END:STATE_MANAGEMENT -->

<!-- BEGIN:EDGE_CASES -->
## Edge Cases

### Hardcoded Values

| Value | Location | Description |
|-------|----------|-------------|
| `200` ms | `SlideWatcher::new()` | Debounce window duration |
| `"md"` | `poll_changes()` | File extension filter |

### Error Handling

| Scenario | Behavior |
|----------|----------|
| Filesystem watching unavailable | `new()` returns `Err`; `app.rs` logs warning and sets `watcher` to `None` |
| Debouncer reports an error | Logged to stderr via `eprintln!("[watcher] error: {err}")` |
| Watched directory deleted | Watcher silently stops producing events |

### Graceful Degradation

If `SlideWatcher::new()` fails (e.g., on platforms without filesystem notification support, or if the directory does not exist at construction time), the app continues to function without hot-reload. The watcher field is `Option<SlideWatcher>`, and the event loop skips the watcher check when it is `None`.
<!-- END:EDGE_CASES -->

<!-- BEGIN:USAGE_EXAMPLE -->
## Usage Example

```rust
use slidet::watcher::SlideWatcher;
use std::path::Path;

// Create a watcher for a slide directory
let mut watcher = SlideWatcher::new(Path::new("examples/01-text-lecture"))?;

// In a polling loop (non-blocking)
loop {
    if watcher.poll_changes() {
        println!("Slides changed, reloading...");
    }
    // Do other work (render UI, handle input, etc.)
}
```
<!-- END:USAGE_EXAMPLE -->

## Tests

Two unit tests in `src/watcher.rs`:

1. **`watcher_detects_new_md_file`**: Verifies that creating a new `.md` file triggers a change detection.
2. **`watcher_ignores_non_md_files`**: Verifies that creating a `.txt` file does not trigger a change detection.

Both tests use temporary directories with 350ms sleep to allow the debouncer to settle.
