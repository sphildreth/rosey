# ADR-0006: Terminal-native parity and safety decisions

Status: Accepted  
Date: 2026-05-23

## Context

Rosey Rust is replacing a Python/PySide6 desktop UI with a Ratatui terminal UI. Some Python behaviors depend on modal dialogs and desktop window affordances that do not translate directly to a terminal without making destructive file operations less predictable.

## Decision

The Rust TUI will preserve the Python behavior and data model, but it will use terminal-native interaction patterns for ambiguous or destructive operations:

- TUI scan/plan identification skips ffprobe duration checks for performance, matching the Python scan worker's `skip_duration=True` path. CLI single-file identification and CLI `run` keep duration checks enabled unless explicitly using the fast path.
- Provider-backed identification is available in the TUI manual identify overlay with `F5` search when online providers are configured. Path-embedded `[tmdbid-*]` values are only treated as confirmed metadata after a provider or cache lookup succeeds.
- A configured `ask` conflict policy blocks a terminal move when destination conflicts exist and asks the user to choose `Skip`, `Replace`, or `Keep Both` before retrying. The terminal UI does not open one modal dialog per conflicting file.
- Cleanup after successful live moves is an explicit Transfer Queue command (`x`) rather than being attached to a desktop dialog's "close and clear" button.
- The Logs / Recovery screen exposes journal inspection for incomplete transfers. Automated recovery remains behind the filesystem journal API and tests rather than an implicit UI action.
- CLI live moves remain opt-in with `--no-dry-run` even if config contains `behavior.dry_run = false`.

## Consequences

Positive:

- Destructive behavior stays explicit and easy to audit in a terminal.
- Long scans stay responsive and avoid one ffprobe process per video during interactive planning.
- Unconfirmed path tags do not inflate confidence scores.
- The TUI can be used over SSH and in small terminals without desktop modal workflows.

Negative:

- The terminal UI is not pixel-for-pixel PySide6 parity.
- Conflict handling is coarser than the Python per-file conflict dialog.
- Cleanup is one extra explicit command after a live move.
- Terminal-level snapshot/input-flow tests are still needed to lock down the interaction contract.

## Alternatives considered

- Recreate every PySide6 modal flow in terminal popups: rejected because it creates awkward keyboard traps and weakens safety for destructive moves.
- Treat `[tmdbid-*]` as authoritative without provider confirmation: rejected because Python only places the path ID into item metadata after a successful TMDB lookup.
- Run duration probing during TUI planning: rejected because Python intentionally skips duration during scan for performance, and the TUI should remain responsive for large libraries.
