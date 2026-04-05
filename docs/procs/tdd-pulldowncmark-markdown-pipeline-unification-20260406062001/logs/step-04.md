## Step 4 — Add a regression sample for links and task lists
### Actions Taken
- Added `examples/05-parser-edge-cases/04-links-and-tasks.md`.
- Covered one inline link, one completed task item, one incomplete task item, and a trailing paragraph to protect mixed-content rendering.

### Verify Result
- Ran `rg -n 'pulldown-cmark|\\[x\\]|\\[ \\]' examples/05-parser-edge-cases/04-links-and-tasks.md`.
- Result: matched the expected link and both task markers.
- Timestamp: 2026-04-05T22:33:55Z
