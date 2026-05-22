---
title: "Lesson: Terminal Link Rendering in Ratatui — OSC 8 and URL Detection"
doc_type: lesson
brief: "OSC 8 escape sequences embedded in ratatui Span content break layout; rendering the URL as visible text lets terminals auto-detect and make it clickable"
confidence: verified
created: 2026-05-23
updated: 2026-05-23
revision: 1
---

# Lesson: Terminal Link Rendering in Ratatui — OSC 8 and URL Detection

## Context

`slidet` renders Markdown slides in the terminal using `ratatui`. Markdown links like
`[label](https://example.com)` need to be both visually distinct and actionable. The
natural first approach was to use OSC 8 terminal hyperlink escape sequences.

## Problem

**OSC 8 escape sequences embedded in ratatui `Span` content** break the terminal layout.

When you embed an OSC 8 sequence like `\x1b]8;;https://example.com\x1b\\label\x1b]8;;\x1b\\`
inside a `Span::raw("...")` or `Span::styled("...", ...)`, ratatui's cell-based grid layout
treats every byte of the escape sequence as a display character. Characters like `]`, `;`,
`\`, `8` etc. within the escape sequence each occupy one cell, pushing subsequent content
out of alignment and creating garbled output.

This happens because ratatui's layout pipeline operates on the text content of `Span`
objects without interpreting terminal escape sequences. The framework counts every
character (visible or invisible) toward column width.

## Solution

A two-layer approach:

### Layer 1: Visible URL + terminal auto-detection

Render links as `label (url)` where:
- The **label** is styled in light blue with underline (`Color::LightBlue` + `Modifier::UNDERLINED`)
- The **URL** is displayed in dark gray (`Color::DarkGray`)

Modern terminals (iTerm2, Kitty, WezTerm, Terminal.app, Windows Terminal) automatically
detect URLs in visible text and make them Cmd+Click (or Ctrl+Click) actionable. By showing
the URL as text, ratatui counts the correct number of cells, and the terminal's URL
detection handles the interactivity.

```rust
// In render_inline_span() — src/ui.rs
markdown::InlineSpan::Link { label, destination } => {
    vec![
        Span::styled(
            label.clone(),
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::styled(
            format!(" ({destination})"),
            Style::default().fg(Color::DarkGray),
        ),
    ]
}
```

The dark gray color for the URL makes it visually subdued while remaining machine-readable
for the terminal's URL detector.

### Layer 2: Keyboard shortcut fallback

Added the `o` keybinding (`KeyCode::Char('o')`) to open all links from the current slide
in the system browser using the `open` crate (v5). This is the fallback for situations
where Cmd+Click is unavailable or the terminal doesn't support URL detection:

```rust
// In App::open_link_for_slide() — src/app.rs
fn open_link_for_slide(&self) {
    // Resolves raw_markdown for current slide based on Mode::Browse or Mode::Present
    let links = crate::markdown::collect_links(markdown);
    for url in &links {
        if let Err(e) = open::that(url) {
            eprintln!("[open] failed to open {url}: {e}");
        }
    }
}
```

A new `collect_links()` function in `src/markdown.rs` extracts all unique URLs from the
structured Markdown block tree, traversing headings, paragraphs, lists, quotes, and tables.

## Why OSC 8 Doesn't Work with Ratatui

OSC 8 is the ANSI standard for terminal hyperlinks:

```
ESC ] 8 ; ; URL BEL text ESC ] 8 ; ; BEL
```

Where `ESC` is `\x1b` and `BEL` is `\x07` or `\x1b\\`. Modern terminals that support OSC 8:
- Parse the escape sequence at the terminal emulator level
- Apply the URL to the visible text without the escape chars taking up cells
- Handle Cmd+Click on the visible text

However, **ratatui does not pass through escape sequences to the terminal as-is**. Instead,
ratatui builds a cell-based grid where each character maps to one cell. Any escape
characters embedded in `Span` content are counted as display characters in the grid,
breaking column alignment.

The ratatui rendering model separates concerns:
- **Styling** is done through `Style` objects (colors, modifiers)
- **Content** is plain text (or styled text via `Span`)

Terminal escape sequences are not part of ratatui's content model, and there is no
ratatui-native API for OSC 8 hyperlinks.

## Benefits

1. **Layout correctness**: No invisible escape characters counted as display width
2. **Broad compatibility**: Terminal URL detection works in most modern terminals
3. **Visual clarity**: Users can see the URL before clicking
4. **Keyboard fallback**: `o` key works even without mouse support or URL detection
5. **Debuggable**: The actual URL text appears in `cargo test` output and debug rendering

## Trade-offs

1. **Visual noise**: URLs take up screen space (mitigated by dark gray styling)
2. **Extra keybinding**: Users need to learn the `o` key for mouse-less link opening
3. **No OSC 8 integration**: If ratatui adds native OSC 8 support in the future, this
   approach would be partially redundant
4. **URL clutter in browse preview**: The right-side preview panel in Browse mode shows
   URLs inline, which can look busy on link-heavy slides

## When to Consider OSC 8

OSC 8 is viable in ratatui when you control the output at the **terminal backend level**
rather than the Span level. For example, writing raw escape sequences to stdout *between*
ratatui draw calls would work because those bytes go directly to the terminal without
going through ratatui's cell grid. However, this approach:
- Cannot associate links with specific ratatui-rendered text
- Requires coupling your code to the terminal backend
- Would need to be coordinated with ratatui's frame rendering

## Related Changes

- `src/ui.rs`: Link rendering — label in LightBlue+underline, URL in DarkGray
- `src/markdown.rs`: `collect_links()`, `block_links()`, `list_item_links()`, `inline_links()`
- `src/app.rs`: `open_link_for_slide()` method, `KeyCode::Char('o')` handler
- `Cargo.toml`: Added `open = "5"` dependency

## References

- Ratatui Span API: https://docs.rs/ratatui/latest/ratatui/text/struct.Span.html
- OSC 8 specification: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
- `open` crate: https://crates.io/crates/open (cross-platform `open::that()`)
