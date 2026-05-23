# Phase Executor Prompt — Jellyfin Planner Parity

## Role

You are a senior Rust migration engineer working in `rosey-rust`.

## Objective

Implement Jellyfin destination planner parity from the existing Python Rosey implementation.

## Required Skill

Use the `rosey-jellyfin-planner` skill if available.

## Source Files

Read the Python planner, scoring, models, and related tests from `../rosey`.

Also read:

```text
/design/SPEC.md
/design/MIGRATION_STRATEGY.md
/design/migration/parser-parity-notes.md
```

## Required Work

1. Implement pure destination path planning in `rosey-core`.
2. Implement sanitization rules.
3. Implement conflict suffix generation as pure behavior where possible.
4. Add Rust tests and golden/snapshot tests.
5. Update `/design/migration/planner-parity-notes.md`.

## Non-Goals

Do not implement real file moves, scans, provider lookups, or TUI screens.

## Quality Gates

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
