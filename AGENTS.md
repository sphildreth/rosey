# AGENTS.md — Rosey Rust

This file is for coding agents working on Rosey Rust.

## Prime directive

Rosey Rust is a module-by-module rewrite of the existing Python/PySide6 Rosey project.

Do not invent behavior when the Python implementation or tests already define it. Preserve useful behavior by reading the existing repo, extracting fixtures/golden outputs, and implementing Rust parity.

Expected local sibling layout:

```text
../rosey        existing Python implementation
../rosey-rust   this repository
```

## Repository conventions

- `/docs` is user-facing documentation.
- `/design` is for product/design/specification material, ADRs, prompts, and migration plans.
- `/design/adr` contains ADRs.
- `/design/prompts` contains reusable coding-agent prompts.
- Rust code lives under `/crates`.
- Fixture and golden test data lives under `/tests`.

## Required workflow for migration work

For each migrated behavior:

1. Identify the relevant Python module and tests in `../rosey`.
2. Summarize the behavior in the PR/commit message.
3. Add or update fixtures/golden files when appropriate.
4. Implement only the corresponding Rust module.
5. Add Rust tests.
6. Run:
   - `cargo fmt --all`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
7. Update design docs if behavior or architecture changes.
8. Add an ADR for significant architecture changes.

## Scope control

Do not rewrite unrelated crates while doing a focused migration task.

Good task:

> Port filename year and episode parsing from Python `patterns.py` into `rosey-core::patterns`, including parity tests.

Bad task:

> Port patterns, scanner, mover, metadata, and TUI all at once.

## Safety rules for filesystem work

File move/copy code is high risk.

Never implement destructive behavior without:

- dry-run tests
- temp-dir integration tests
- conflict-policy tests
- rollback/journal tests
- clear error handling
- explicit user confirmation in TUI/CLI before execute mode

Default behavior must be dry-run unless the command clearly requests execution.

## Testing expectations

Use:

- Rust unit tests for pure behavior
- integration tests for filesystem behavior
- `insta` for golden JSON snapshots
- `proptest` for parser/sanitizer properties where useful
- temp directories for mover tests

Do not remove Python-derived golden files unless the design docs and ADRs explain why behavior intentionally changed.

## Documentation expectations

Update `/docs` for user-facing behavior.

Update `/design` for implementation strategy, prompts, and architectural decisions.

Use ADRs for meaningful decisions:

```bash
./scripts/new-adr.sh "Decision title"
```

## Rust style

- Prefer small, explicit modules.
- Prefer typed domain models over loose maps.
- Use `thiserror` for library errors.
- Use `anyhow` at binary boundaries.
- Use `tracing` for structured logs.
- Keep UI crates thin. The core engine must not depend on the TUI.
- Avoid `unsafe` unless an ADR explicitly approves it.
