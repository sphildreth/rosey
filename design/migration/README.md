# Rosey Rust Migration Notes

This directory tracks behavior parity between the Python Rosey implementation and the Rust rewrite.

Create one file per migration slice:

```text
parser-parity-notes.md
identifier-parity-notes.md
planner-parity-notes.md
scanner-parity-notes.md
mover-parity-notes.md
metadata-parity-notes.md
tui-implementation-notes.md
```

Each note should include:

```text
Python source files referenced
Rust modules implemented
Tests/golden fixtures added
Intentional deviations
Release validation notes
Recommended next step
```
