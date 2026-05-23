# Agent Output Discipline

Keep changes narrow.

Do not:

- Reformat unrelated files.
- Rename crates or directories without explicit task scope.
- Add dependencies without explaining why.
- Implement future phases early.
- Mix TUI work into core migration slices.
- Hide failures or skipped test commands.

At task completion, report:

```text
Files changed
Behavior implemented
Tests added/updated
Commands run
Known gaps
Recommended next step
```
