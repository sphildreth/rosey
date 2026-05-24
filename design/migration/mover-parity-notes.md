# Mover / Transfer Engine Parity Notes

## Status
Complete. Python `move_file_transactional`, `move_with_sidecars`, `check_preflight`, `same_volume`, and `apply_conflict_suffix` ported to `crates/rosey-fs/src/mover.rs`.

## Ported Functions

| Python | Rust | Notes |
|--------|------|-------|
| `move_file_transactional` | `move_file_transactional` | Dry-run, same-volume rename, cross-volume copy-verify-delete |
| `move_with_sidecars` | `move_with_sidecars` | Transactional move with sidecar rollback |
| `check_preflight` | `check_preflight` | Space, permissions, path-length checks |
| `same_volume` | `same_volume` | Unix `st_dev` comparison; Windows returns `false` conservatively |
| `apply_conflict_suffix` | `apply_conflict_suffix` | `(1)`, `(2)` suffix for `KeepBoth` |
| `discover_sidecars` | `crate::discover_sidecars` | Reuses already-ported sidecar discovery |

## Behavior Verified

- **Dry-run mode**: no files touched; returns `WouldMove`
- **Same-volume move**: uses `fs::rename` (atomic)
- **Cross-volume move**: `fs::copy` → size check plus Python-style large-file byte verification → `fs::remove_file` source
- **Skip policy**: destination exists → `Skipped`, source untouched
- **Replace policy**: destination exists → overwritten, source removed
- **KeepBoth policy**: destination exists → `(1)`, `(2)` suffix applied
- **Parent directory creation**: `fs::create_dir_all` before move
- **Missing source**: returns `Err(MoveError::SourceMissing)`
- **Sidecar move**: sidecars renamed to match destination stem
- **Transactional rollback**: if any sidecar fails, all already-moved files are deleted
- **Preflight checks**:
  - Destination directory created if missing
  - Writability tested via temporary probe file
  - Free space checked via `statvfs` (Unix) with 100 MiB buffer
  - Path length checked against 255-char limit

## Intentional Deviations

1. **No `copy_file_range` fast path**: Python tries Linux `copy_file_range` for zero-copy cross-volume copies. Rust uses `fs::copy` which delegates to the OS and may use `copy_file_range` internally on modern Rust std.
2. **Operation journal is a Rust safety enhancement**: Python does not journal. Rust records move, copy, verification, source-delete, completion, rollback, and failure entries for TUI recovery inspection.
3. **Cancellation-safe state transitions are journal-backed**: Rust records durable transfer progress, but does not currently expose automatic resume/rollback from the TUI.
4. **Windows same-volume detection**: Returns `false` conservatively, meaning Windows always uses copy-verify-delete.
5. **No logging inside mover**: Python emits `logger.info`/`logger.error`. Rust returns structured errors and journal entries; callers can log.

## Safety Compliance

Per `rosey-file-engine-safety` skill requirements:

- ✅ Dry-run mode available (`dry_run: bool`)
- ✅ Explicit conflict policy (`Skip`, `Replace`, `KeepBoth`)
- ✅ Temp-dir integration tests (all mover tests use `tempfile::tempdir()`)
- ✅ Source deletion only after destination verification (size check and large-file byte-content check)
- ✅ Partial copy cleanup on verification failure (`fs::remove_file`)
- ✅ Rollback on sidecar failure (deletes already-moved files)
- ✅ Never tests with real user media paths

## Test Coverage

17 tests in `crates/rosey-fs/tests/mover_tests.rs`:

- `same_volume_detects_same_device`
- `apply_conflict_suffix_increments`
- `move_file_dry_run`
- `move_file_same_volume`
- `move_file_skip_existing`
- `move_file_replace_existing`
- `move_file_keep_both`
- `move_file_creates_parent_dirs`
- `move_file_source_missing`
- `verify_file_copy_accepts_identical_files`
- `verify_file_copy_rejects_same_size_different_content`
- `move_with_sidecars_moves_all_files`
- `move_with_sidecars_dry_run`
- `move_with_sidecars_rollback_on_sidecar_failure`
- `preflight_all_ok`
- `preflight_creates_destination`
- `preflight_detects_long_path`

All tests use `tempfile::tempdir()` — no real user media paths touched.

## Dependencies Added

- `libc = "0.2"` to workspace root `Cargo.toml` (for Unix `statvfs`)
- `[target.'cfg(unix)'.dependencies]` `libc.workspace = true` in `crates/rosey-fs/Cargo.toml`

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace` (146 passed total)
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Step

Mover parity is complete. Add new temp-dir regression tests here for any future filesystem edge cases before changing destructive behavior.
