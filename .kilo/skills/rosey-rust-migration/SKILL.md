---
name: rosey-rust-migration
description: Use when converting existing Python Rosey behavior into the Rust rewrite one module or migration slice at a time.
---


# Rosey Rust Migration Skill

Use this skill when porting behavior from `../rosey` into `rosey-rust`.

## Workflow

1. Identify the exact Python source files and tests that define the behavior.
2. Read only the relevant files.
3. Write down the behavior contract before coding.
4. Implement the Rust equivalent in the correct crate.
5. Add Rust tests.
6. Run the Rust quality gates.
7. Update `/design/migration/` notes with coverage and gaps.

## Do Not

- Do not rewrite the entire application.
- Do not mutate `../rosey`.
- Do not mix unrelated phases.
- Do not implement the TUI before the core/CLI path is testable.

## Preferred Migration Order

```text
1. parser/patterns
2. Jellyfin destination planner
3. NFO parsing
4. scanner
5. sidecar discovery
6. dry-run transfer planning
7. move/copy engine
8. journal/recovery
9. CLI
10. Ratatui TUI
11. metadata providers/cache
```

## Required Completion Report

```text
Python files referenced:
Rust files changed:
Tests added:
Quality gates run:
Known parity gaps:
Next recommended slice:
```
