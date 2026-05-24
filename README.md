<p align="center">
  <img src="./graphics/logo.png" alt="Rosey logo" width="160" />
</p>

<h1 align="center">Rosey</h1>

<p align="center"><b>Media organizer for Jellyfin</b> - scan, identify, preview, and safely organize Movies and TV Shows into Jellyfin-friendly folders.</p>

<p align="center">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-green"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%26%20Linux-8A2BE2">
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.82%2B-orange">
  <img alt="UI" src="https://img.shields.io/badge/UI-Ratatui-41b883">
</p>

<p align="center">
  <a href="#features">Features</a> |
  <a href="#documentation">Documentation</a> |
  <a href="#status">Status</a> |
  <a href="#developer-setup">Developer setup</a> |
  <a href="#contributing">Contributing</a> |
  <a href="#license">License</a>
</p>

---

Rosey scans a source folder, identifies Movies and TV Shows from offline signals and optional TMDB/TVDB metadata, previews Jellyfin-ready destinations, and moves selected items with dry-run defaults, preflight checks, sidecar handling, rollback, and operation journals.

Privacy-first: Rosey has no telemetry, and online provider calls happen only when configured and enabled.

## Features

- **Offline identification** from filenames, folder structure, season folders, dates, multipart markers, and `.nfo` files
- **Optional online metadata** with TMDB primary and TVDB support, DecentDB caching, TTL expiry, and rate limiting
- **Provider-confirmed path IDs** for `[tmdbid-*]` paths without treating unconfirmed path tags as trusted metadata
- **Confidence scoring** with reasons and configurable Green/Yellow/Red thresholds
- **Jellyfin naming conventions** for Movies and TV Shows
- **Safe batch moves** with dry-run mode, same-volume rename, cross-volume copy-verify-delete, conflict policies, rollback, and JSON Lines journals
- **Sidecar and companion handling** for subtitles, `.nfo` files, and artwork, including symlinked sidecar files
- **Configurable cleanup** from the TUI for auto-delete patterns and empty source directories after successful live moves
- **Terminal UI** with dashboard, scan results, plan preview, transfer queue, logs/recovery, settings, doctor, and help screens
- **Manual identification** with terminal-native edits and provider search when providers are enabled
- **CLI automation** for scan, identify, doctor, and full run workflows, including stable JSON output

## Documentation

- **Setup**: [docs/SETUP.md](./docs/SETUP.md) - build and run commands
- **User Guide**: [docs/USER_GUIDE.md](./docs/USER_GUIDE.md) - workflow and naming goals
- **Configuration**: [docs/CONFIGURATION.md](./docs/CONFIGURATION.md) - `rosey.json` paths and settings
- **Safety Model**: [docs/SAFETY_MODEL.md](./docs/SAFETY_MODEL.md) - dry-run, journal, and move safety design
- **Troubleshooting**: [docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) - common issues
- **Docs Index**: [docs/README.md](./docs/README.md) - user-facing documentation overview
- **Architecture Decisions**: [design/adr/README.md](./design/adr/README.md) - ADR index

Quality gates:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Optional release validation can include live TMDB/TVDB smoke tests with user-provided API keys. The default test suite uses local cache-backed tests and does not require network access.

## Developer Setup

**Requirements**

- Rust 1.82 or newer
- Cargo
- A terminal with ANSI support; Windows Terminal is recommended on Windows
- `ffprobe` available on `PATH` for duration checks when movie duration validation is enabled

### Build

```bash
cargo build --workspace
```

For release binaries:

```bash
cargo build --release --workspace
```

### Run the TUI

```bash
cargo run -p rosey-tui -- /path/to/source /movies /tv
```

The TUI also reads configured paths from `rosey.json`, so positional paths can be omitted after configuration.

### Run the CLI

```bash
# Scan
cargo run -p rosey-cli -- scan /path/to/source --json

# Identify one file
cargo run -p rosey-cli -- identify "Example.Show.S01E02.mkv" --json

# Check configuration and system readiness
cargo run -p rosey-cli -- doctor

# Full dry-run workflow
cargo run -p rosey-cli -- run /path/to/source --movies-target /movies --tv-target /tv --dry-run

# Execute live moves explicitly
cargo run -p rosey-cli -- run /path/to/source --movies-target /movies --tv-target /tv --no-dry-run
```

CLI live moves require `--no-dry-run`; a saved `behavior.dry_run = false` does not make the CLI destructive by default.

### Configuration

Rosey reads `rosey.json` from the platform config directory:

- Linux/macOS: `$XDG_CONFIG_HOME/rosey/rosey.json`, or `~/.config/rosey/rosey.json`
- Windows: `%APPDATA%\rosey\rosey.json`

The terminal UI reads `ui.theme` from the config to control TUI colors. Supported values are:

- `default`
- `terminal`
- `high_contrast`
- `no_color`
- `rainbow`

Unknown values fall back to `default`; the Doctor check can report this configuration issue.

The CLI can persist explicitly supplied path arguments:

```bash
cargo run -p rosey-cli -- run /path/to/source --movies-target /movies --tv-target /tv --save-config
```

The TUI Settings screen can edit and save paths, dry-run behavior, symlink scanning, conflict policy, confidence thresholds, provider settings, cache TTL, and cleanup patterns. The Doctor screen is available as tab `7` and can refresh checks with `o`.

### Run Tests

```bash
cargo test --workspace
```

### Run Quality Checks

```bash
# All checks
just check

# Individual checks
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Repository Layout

```text
crates/
  rosey-core/        domain models, parser, planner, identifier, scorer, config, grouper
  rosey-fs/          scanner, sidecar discovery, mover, operation journal
  rosey-metadata/    TMDB/TVDB providers, DecentDB cache, rate limiter
  rosey-cli/         command-line interface with JSON output
  rosey-tui/         Ratatui/Crossterm terminal UI
docs/                user-facing documentation
design/              PRD, SPEC, ADRs, prompts, planning notes
tests/
  fixtures/          media trees, sidecars, NFO, conflict fixtures
  golden/            golden JSON parity outputs
```

## Testing

- Unit tests for parser, planner, NFO parsing, identifier logic, scorer, config, and provider cache
- Integration tests for scanner, mover, sidecars, companions, rollback, journal, and TUI render smoke coverage
- Property-based tests for parser, sanitizer, planner, scorer, and identifier no-panic guarantees
- Golden output tests for behavior stability
- Temp-directory filesystem tests for destructive-operation safety

## Contributing

Using AI coding agents? Start with [AGENTS.md](./AGENTS.md). It defines repository conventions, safety requirements, and validation expectations.

Contributions are welcome. A few tips:

- Keep UI crates thin; shared behavior belongs in `rosey-core`, `rosey-fs`, or `rosey-metadata`
- Add or update tests for behavior changes, especially filesystem operations
- Update docs when user-visible behavior or architecture changes
- Run the full quality gates before opening a PR

## License

MIT - see [LICENSE](./LICENSE).
