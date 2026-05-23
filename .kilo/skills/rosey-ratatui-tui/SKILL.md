---
name: rosey-ratatui-tui
description: Use when implementing or reviewing Ratatui/Crossterm terminal UI screens, keyboard navigation, layout, event loop, and progress rendering.
---


# Rosey Ratatui TUI Skill

Use this skill for terminal UI work only after the relevant core/CLI behavior is testable.

## Rust Target

```text
crates/rosey-tui/
```

## TUI Principles

- TUI renders state; it does not own business logic.
- Core and filesystem crates emit events.
- Keep keyboard navigation visible and discoverable.
- Prefer ANSI-rich, power-user-friendly layouts.
- TUI should be usable over SSH.
- Do not block the UI loop with scan/copy/metadata work.

## Suggested Screens

```text
Dashboard
Scan Results
Plan Preview
Transfer Queue
Logs / Recovery
Settings
Help
```

## Required Patterns

- Central `AppState`.
- Explicit `Screen` enum.
- Input handling separated from rendering.
- Background work communicates through channels/events.
- Tests for state transitions where practical.

## Do Not

- Do not put file moving logic inside TUI code.
- Do not add metadata provider behavior in the TUI crate.
- Do not build screens before commands/events exist in core/CLI.
