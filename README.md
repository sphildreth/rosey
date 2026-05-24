# Rosey Rust

Rosey Rust is a fast, robust, cross-platform media organizer for Jellyfin users. It's a rewrite of the Python/PySide6 Rosey application, built in Rust for performance, reliability, and simplified distribution.

The goal is to provide:

- a Rust core engine for scanning, identifying, planning, and moving media files
- a CLI for automation, testing, and parity verification
- a Ratatui/Crossterm TUI for the main interactive experience
- golden-master parity testing against the existing Python repo
- safe file operations with dry-run, preflight, journaling, rollback/recovery, and clear logs

## Quick start

```bash
# CLI
cargo run -p rosey-cli -- scan /path/to/media --json
cargo run -p rosey-cli -- identify "Example.Show.S01E02.mkv" --json
cargo run -p rosey-cli -- run /path/to/media --movies-target /movies --tv-target /tv --dry-run

# TUI
cargo run -p rosey-tui /path/to/media /movies /tv
```

## Repository layout

```text
crates/
  rosey-core/        domain models, parser, planner, identifier, scorer, config, grouper
  rosey-fs/          scanner, sidecar discovery, mover, operation journal
  rosey-metadata/    TMDB/TVDB providers, SQLite cache, rate limiter
  rosey-cli/         command-line interface (scan, identify, run) with JSON output
  rosey-tui/         Ratatui/Crossterm terminal UI (7 screens, keyboard-first)
docs/                user-facing documents
design/              PRD, SPEC, ADRs, migration prompts
tests/
  fixtures/          test fixture trees and sample files
  golden/            golden JSON outputs and parity tests
```

## Features

### Scanner
- Recursive directory scanning for video files (mkv, mp4, avi, mov, wmv, flv, m4v, mpg, mpeg, webm, ts)
- Symlink control, error reporting, concurrent traversal

### Identifier
- Filename pattern parsing: SxxEyy, 1x02, Season/Episode, date-based, multipart
- Year, date, episode, part, and TMDB ID extraction
- Title cleanup (quality tags, codecs, release groups, descriptors)
- NFO file parsing (XML) for title, year, IDs, season/episode
- Provider-confirmed TMDB path IDs when online providers are configured
- Duration-aware movie rejection through ffprobe for non-fast identification paths
- Companion file discovery (subtitles, images)

### Planner
- Jellyfin-compatible destination path generation
- Movie layout: `Movies/Title (Year)/Title (Year).mkv`
- TV layout: `TV/Show/Season XX/Show - SxxEyy - Title.mkv`
- Path sanitization for cross-platform compatibility
- Conflict suffix generation (KeepBoth policy)

### Mover
- Dry-run by default
- Preflight checks (free space, writability, path length)
- Same-volume atomic rename
- Cross-volume copy with size verification before source deletion
- Sidecar file movement (subtitles, images, NFO)
- Rollback on partial failure
- JSON Lines operation journal for crash recovery
- Conflict policies: Skip, Replace, KeepBoth

### CLI
- `scan` — recursive video file discovery with JSON output
- `identify` — config-aware identification for a single file
- `run` — scan, identify, plan, and optionally move in one command
- `--save-config` — persist explicitly supplied path arguments
- Stable JSON output for scripting and parity testing

### TUI
- 7 screens: Dashboard, Scan Results, Plan Preview, Transfer Queue, Logs/Recovery, Settings, Help
- Keyboard-first navigation with discoverable shortcuts
- Sortable/filterable plan preview with confidence bands
- Manual identify overlay with provider search when configured
- Editable settings screen with config persistence
- Confirmation dialog before destructive operations
- Responsive scan, plan, and move progress feedback
- Journal inspection and explicit post-move cleanup commands

### Metadata (optional)
- TMDB and TVDB providers with rate limiting
- SQLite-based cache with configurable TTL
- Fully offline-capable — no provider required for basic organization

## Quality gates

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Testing

- Unit tests for parser, planner, scanner, mover, sidecars, companions, NFO
- Integration tests with tempdir for mover and scanner
- Property-based tests (proptest) for parser/sanitizer no-panic guarantees
- Golden parity tests comparing Rust output against Python reference
- Test fixtures: media trees, sidecar collections, NFO files, conflict scenarios

## Migration status

Core, CLI, metadata, and the terminal application now implement the main Python Rosey workflows. Remaining cutover work is concentrated in terminal-level UI smoke/snapshot tests, optional real-provider smoke tests with user-supplied API keys, and repository release tasks. See `design/MIGRATION_PLAN.md` for current status.

## Reference

The Python Rosey reference implementation lives at `../rosey` and is tagged with `python-baseline`.

Design documents:
- `design/PRD.md` — Product Requirements Document
- `design/SPEC.md` — Technical Specification
- `design/PARITY_STRATEGY.md` — Parity testing strategy
- `design/MIGRATION_PLAN.md` — Full migration plan with status
- `design/adr/` — Architecture Decision Records
