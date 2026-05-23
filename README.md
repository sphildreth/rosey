# Rosey Rust

Rosey Rust is the planned Rust rewrite of the existing Python/PySide6 Rosey media organizer.

The goal is to build a fast, robust, cross-platform media organizer for Jellyfin users with:

- a Rust core engine
- a CLI for automation, testing, and parity verification
- a Ratatui/Crossterm TUI for the main interactive experience
- golden-master parity testing against the existing Python repo
- safe file operations with dry-run, journaling, rollback/recovery, and clear logs

## Repository strategy

This repository is intended to live beside the existing Python repo during migration:

```text
~/github/rosey        # existing Python/PySide6 implementation
~/github/rosey-rust   # this Rust rewrite
```

The Python repo remains the behavioral reference until the Rust CLI and TUI prove parity on the important behavior.

## Layout

```text
crates/
  rosey-core/        domain models, parsing, planning primitives
  rosey-fs/          scanner, sidecar discovery, transfer engine
  rosey-metadata/    metadata provider interfaces and cache boundary
  rosey-cli/         command-line interface and parity harness
  rosey-tui/         Ratatui/Crossterm terminal interface
docs/                user-facing documents
design/              PRD, SPEC, ADRs, migration prompts, agent guidance
tests/
  fixtures/          input fixture trees/files
  golden/            expected JSON outputs from Python Rosey v1
```

## First commands

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

cargo run -p rosey-cli -- scan . --json
cargo run -p rosey-cli -- identify "Example.Show.S01E02.mkv" --json
cargo run -p rosey-tui
```

## Migration rule

No migrated Rust feature is considered done until it either:

1. passes direct Rust unit/integration tests, or
2. matches a Python-generated golden output from the existing Rosey repo.

See:

- `design/MIGRATION_PLAN.md`
- `design/PARITY_STRATEGY.md`
- `design/prompts/MASTER_MIGRATION_PROMPT.md`

## Status

Starter repository only. It contains a compiling-oriented skeleton, early domain models, placeholder CLI/TUI entry points, ADRs, and coding-agent prompts.
