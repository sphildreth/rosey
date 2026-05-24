# TUI Implementation Notes

## Status

Partial. The Rust TUI now reads supported config defaults, keeps long-running scan/plan/move work off the render loop, and exposes progress/status feedback across the dashboard and workflow screens.

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
- CLI positional arguments still override configured paths.
- Scan, plan, and move operations run through background workers.
- Scan results stream into the Scan Results screen as files are discovered.
- Dashboard, scan results, plan preview, transfer queue, and status bar show current operation state.
- Live move execution requires confirmation.
- Dry-run move preview reports `WOULD MOVE` instead of implying files were moved.

## Known Gaps

- Settings screen is read-only; it does not edit or save config yet.
- Metadata providers are not wired into the TUI identification flow.
- Duration-aware scoring is not implemented.
- Terminal-level screen snapshots and input-flow tests are not yet present.
- Logs/recovery screen does not expose journal recovery actions yet.

## Intentional Deviations

- The Rust TUI is keyboard-first and terminal-native, so it does not attempt pixel parity with the Python/PySide6 interface.
- The Rust TUI blocks quit while a background operation is active. This keeps worker state and terminal cleanup simple until cancellable operations are designed.

## Recommended Next Slice

Implement settings editing and `save_config` from the TUI, then wire provider-backed identification through a shared core service used by both CLI and TUI.
