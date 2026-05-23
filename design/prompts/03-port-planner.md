# Prompt: Port Jellyfin Planner

Port Jellyfin destination planning behavior into Rust.

## Python source to inspect

Look in `../rosey` for planner/naming/sanitization tests and docs.

## Rust target

- `crates/rosey-core/src/planner.rs`
- `crates/rosey-core/src/sanitize.rs`
- tests under `crates/rosey-core`

## Requirements

Support:

- movie folder/file naming
- TV show season folder naming
- episode naming
- multi-episode naming
- extension preservation
- invalid character sanitization
- Windows reserved names
- conflict suffix strategy

## Success criteria

- unit tests cover edge cases
- snapshot tests for destination paths
- documented intentional deviations from Python behavior
