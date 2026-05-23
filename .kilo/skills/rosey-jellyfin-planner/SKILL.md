---
name: rosey-jellyfin-planner
description: Use when implementing Jellyfin destination path planning, filename sanitization, conflict suffixing, and movie/show layout behavior.
---


# Rosey Jellyfin Planner Skill

Use this skill when implementing destination planning and Jellyfin naming rules.

## Python Reference

Read the Python planner/naming code and tests from `../rosey`.

Also read:

```text
/design/SPEC.md
/design/MIGRATION_STRATEGY.md
```

## Rust Target

```text
crates/rosey-core/src/planner.rs
crates/rosey-core/src/sanitize.rs
crates/rosey-core/tests/planner_tests.rs
tests/golden/planner/
```

## Behavior to Preserve

- Movie folder and file naming.
- TV show season folder naming.
- Episode naming.
- Multi-episode naming.
- Specials / Season 00 behavior if present.
- Preserve original file extension.
- Sanitize invalid filesystem characters.
- Handle Windows reserved names.
- Collapse unwanted whitespace.
- Keep-both suffixes: `name (1).ext`, `name (2).ext`.

## Rules

- Planner must not touch the filesystem except where explicitly needed for conflict checks.
- Separate pure path generation from filesystem-aware conflict resolution.
- Use snapshot/golden tests for destination path output.
- Update `/design/migration/planner-parity-notes.md`.
