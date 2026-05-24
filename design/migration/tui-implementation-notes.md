# TUI Implementation Notes

## Status

Complete for terminal-native parity. The Rust TUI reads config defaults, can persist the current in-memory settings snapshot back to `rosey.json`, keeps long-running scan/plan/move/provider-search work off the render loop, and exposes progress/status feedback across the dashboard and workflow screens.

## Python Source Files Referenced

- `../rosey/src/rosey/ui/main_window.py`
- `../rosey/src/rosey/ui/settings_dialog.py`
- `../rosey/src/rosey/ui/progress_dialog.py`

## Rust Modules Implemented

- `crates/rosey-tui/src/app.rs`
- `crates/rosey-tui/src/main.rs`
- `crates/rosey-tui/src/renderer.rs`
- Shared behavior from `rosey-core` and `rosey-fs`

## Implemented Behavior

- Reads `paths.source`, `paths.movies`, `paths.tv`, `behavior.dry_run`, `behavior.conflict_policy`, `scanning.concurrency_local`, `scanning.follow_symlinks`, and confidence thresholds from `rosey.json`.
- Reads provider settings for provider-confirmed TMDB ID enrichment and manual online search.
- CLI positional arguments still override configured paths.
- The Settings screen accepts `w` or `s` to save the current runtime snapshot back to `rosey.json`.
- The Settings screen supports `Up/Down` selection and `e` text editing for paths, dry-run, symlink scanning, conflict policy, confidence thresholds, provider settings, cache TTL, and auto-delete patterns.
- Saved values come from the current in-memory state, so CLI overrides and runtime edits are what get written.
- Scan, plan, and move operations run through background workers.
- Scan results stream into the Scan Results screen as files are discovered.
- Plan work uses provider-aware identification and skips duration checks for performance, matching the Python scan worker.
- The Plan Preview identify overlay supports manual title/year/type edits and `F5` online search when providers are enabled.
- Dashboard, scan results, plan preview, transfer queue, and status bar show current operation state.
- Live move execution requires confirmation.
- `ask` conflict policy blocks a move when destination conflicts exist and asks the user to choose a terminal policy before retrying.
- Dry-run move preview reports `WOULD MOVE` instead of implying files were moved.
- Transfer Queue `x` runs explicit cleanup of configured auto-delete patterns and empty source directories after successful live moves.
- Logs / Recovery `r` inspects the last move journal for incomplete transfers.

## Remaining Test Coverage

- Terminal smoke tests cover the dashboard, settings edit overlay, and identify overlay. Broader end-to-end input-flow snapshots are release-hardening work rather than a Python feature gap.

## Intentional Deviations

- The Rust TUI is keyboard-first and terminal-native, so it does not attempt pixel parity with the Python/PySide6 interface.
- The Rust TUI blocks quit while a background operation is active. This keeps worker state and terminal cleanup simple until cancellable operations are designed.
- The terminal `ask` conflict policy is resolved before the move begins rather than one desktop dialog per conflicting file.
- Post-move cleanup is explicit (`x`) instead of being coupled to a desktop "close and clear" button.

## Recommended Next Step

Expand terminal snapshot coverage for scan, plan, transfer, settings edit, identify provider search, and recovery flows before a public release.
