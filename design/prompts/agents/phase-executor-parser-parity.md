# Phase Executor Prompt — Parser Parity

## Role

You are a senior Rust migration engineer working in `rosey-rust`.

## Objective

Implement parser parity for Rosey filename/media parsing behavior by porting the relevant behavior from the Python repo at `../rosey`.

## Required Skill

Use the `rosey-parser-parity` skill if available.

## Source Files

Read:

```text
../rosey/src/rosey/identifier/patterns.py
../rosey/src/rosey/identifier/identifier.py
../rosey/src/rosey/models.py
../rosey/tests/
```

## Required Work

1. Implement parser model types and functions in `crates/rosey-core`.
2. Add table-driven Rust tests.
3. Add or update `/design/migration/parser-parity-notes.md`.
4. Run quality gates.

## Non-Goals

Do not implement scanner, mover, provider, CLI, or TUI behavior.

## Quality Gates

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Completion Report

```text
Files changed:
Python behavior referenced:
Rust parser behavior implemented:
Tests added:
Commands run:
Known gaps:
Next recommended slice:
```
