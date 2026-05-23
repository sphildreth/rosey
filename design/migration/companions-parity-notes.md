# Companion File Discovery Parity Notes

## Status
Complete. Python `Identifier._discover_companion_files` behavior ported to `crates/rosey-core/src/companions.rs`.

## Ported Functions

| Python | Rust | Notes |
|--------|------|-------|
| `_discover_companion_files` | `discover_companion_files` | Identifier-level companion discovery |
| `subtitle_exts` | `COMPANION_SUBTITLE_EXTS` | `.srt`, `.ass`, `.vtt` |
| `image_exts` | `COMPANION_IMAGE_EXTS` | `.jpg`, `.png`, `.jpeg` |
| `subtitle_folder_names` | `SUBTITLE_FOLDER_NAMES` | `subs`, `sub`, `subtitles`, `subtitle` |
| Recursive `rglob` | `recursive_subtitle_scan` | Recursive directory traversal |

## Behavior Verified

- **Same-directory subtitles**: `.srt`, `.ass`, `.vtt` in the same directory as the media file are discovered
- **Same-directory images**: `.jpg`, `.png`, `.jpeg` in the same directory are discovered
- **Subtitle subdirectories**: `Subs/`, `sub/`, `Subtitles/`, `Subtitle/` recognized case-insensitively
- **Recursive scanning**: deeply nested subtitle files found (e.g., `Subs/English/Forced/en.forced.srt`)
- **Multiple formats**: all 3 subtitle formats discovered from a single folder
- **Mixed locations**: same-directory + subdirectory companions both found
- **Non-subtitle folders ignored**: `Extras/should_not_find.srt` is not discovered
- **Nonexistent parent**: returns empty `Vec`
- **Files without extension**: ignored

## Key Differences from `discover_sidecars`

| | `discover_sidecars` (mover) | `discover_companion_files` (identifier) |
|---|---|---|
| Scope | Same directory only | Same directory + subtitle subdirectories |
| Stem matching | Yes (must match media file stem) | No (any file in directory) |
| Image files | Yes (`.jpg`, `.png`, `.jpeg`) | Yes |
| Subtitle extensions | 10+ (`.srt`, `.ssa`, `.ass`, `.vtt`, `.sub`, `.idx`, etc.) | 3 (`.srt`, `.ass`, `.vtt`) |
| Recursive | No | Yes (into `Subs/` etc.) |

## Known Gaps / Intentional Deviations

1. **No caching**: Python `Identifier` has `_duration_cache`, `_show_folder_cache`, etc. Caching is an optimization to add later.
2. **No logging**: Python silently catches `OSError`/`PermissionError`. Rust does the same via `let Ok(entries) = fs::read_dir(...)`.
3. **No symlink following**: Rust `file_type()` does not follow symlinks. Python `item.is_file()` does. This is a minor semantic difference.

## Test Coverage

15 tests in `crates/rosey-core/tests/companions_tests.rs`:

- `same_directory_subtitle`
- `same_directory_image`
- `same_directory_mixed`
- `subs_folder_basic`
- `subs_folder_case_insensitive`
- `subtitles_folder`
- `sub_folder_variant`
- `subtitle_folder_variant`
- `nested_subtitle_folder`
- `deeply_nested_subtitle_folder`
- `multiple_subtitle_formats`
- `mixed_locations`
- `ignores_other_folders`
- `nonexistent_parent_returns_empty`
- `no_extension_ignored`

All tests use `tempfile::tempdir()` — no real user media paths touched.

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace` (131 passed total)
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Slice

**Move/copy engine parity** — port Python `move_file_transactional` and `move_with_sidecars` from `rosey/mover/mover.py`. This requires:
- Dry-run mode
- Conflict policy handling (`skip`, `replace`, `keep_both`)
- Same-volume rename vs cross-volume copy-verify-delete
- Transactional rollback on failure
- Preflight checks (space, permissions, path length)

Per the file-safety skill, this slice should include an ADR review before implementing destructive operations.
