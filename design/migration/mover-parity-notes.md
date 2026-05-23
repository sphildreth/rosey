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
- **Cross-volume move**: `fs::copy` → size verification → `fs::remove_file` source
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

## Known Gaps / Intentional Deviations

1. **No `copy_file_range` fast path**: Python tries Linux `copy_file_range` for zero-copy cross-volume copies. Rust uses `fs::copy` which delegates to the OS and may use `copy_file_range` internally on modern Rust std.
2. **Verification is size-only**: Python reads large files in chunks to byte-compare. Rust checks size equality only. Full byte-level verification can be added later if needed.
3. **No operation journal**: Python does not journal either; the Rust SPEC mentions JSON Lines journaling as a future feature.
4. **No cancellation-safe state transitions**: Not implemented in this slice.
5. **Windows same-volume detection**: Returns `false` conservatively, meaning Windows always uses copy-verify-delete. A Windows-specific `GetVolumeInformation` implementation can be added later.
6. **No logging inside mover**: Python emits `logger.info`/`logger.error`. Rust returns structured errors; callers can log.

## Safety Compliance

Per `rosey-file-engine-safety` skill requirements:

- ✅ Dry-run mode available (`dry_run: bool`)
- ✅ Explicit conflict policy (`Skip`, `Replace`, `KeepBoth`)
- ✅ Temp-dir integration tests (all mover tests use `tempfile::tempdir()`)
- ✅ Source deletion only after destination verification (size check)
- ✅ Partial copy cleanup on verification failure (`fs::remove_file`)
- ✅ Rollback on sidecar failure (deletes already-moved files)
- ✅ Never tests with real user media paths

## Test Coverage

15 tests in `crates/rosey-fs/tests/mover_tests.rs`:

- `same_volume_detects_same_device`
- `apply_conflict_suffix_increments`
- `move_file_dry_run`
- `move_file_same_volume`
- `move_file_skip_existing`
- `move_file_replace_existing`
- `move_file_keep_both`
- `move_file_creates_parent_dirs`
- `move_file_source_missing`
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

## Recommended Next Slice

**CLI parity** — port the Python CLI commands (`scan`, `identify`, `plan`, `move`) to `rosey-cli`. The core engine now has all the building blocks:
- Scanner (`rosey-fs::scan`)
- Parser / identifier (`rosey-core::patterns`, `rosey-core::nfo`, `rosey-core::companions`)
- Planner (`rosey-core::Planner`)
- Mover (`rosey-fs::move_with_sidecars`)

After CLI parity, the **TUI** (`rosey-tui`) can be built on top of the testable core/CLI path.
