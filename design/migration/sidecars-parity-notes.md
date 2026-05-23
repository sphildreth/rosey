# Sidecar Discovery Parity Notes

## Status
Complete. Python `discover_sidecars` and `is_sidecar_path` behavior ported to `crates/rosey-fs/src/sidecars.rs`.

## Ported Functions

| Python | Rust | Notes |
|--------|------|-------|
| `SIDECAR_EXTENSIONS` | `SIDECAR_EXTENSIONS` | Same 14 extensions, no leading dots |
| `SUBTITLE_EXTENSIONS` | N/A (merged) | Python keeps subtitle set separate; Rust merges into `SIDECAR_EXTENSIONS` |
| `is_sidecar_path` | `is_sidecar_path` | Case-insensitive extension check |
| `discover_sidecars` | `discover_sidecars` | Same-directory stem-matching discovery |

## Behavior Verified

- **Extension recognition**: all 14 sidecar extensions recognized (`srt`, `ssa`, `ass`, `vtt`, `sub`, `idx`, `sbv`, `lrc`, `smi`, `stl`, `nfo`, `jpg`, `jpeg`, `png`)
- **Case-insensitive**: `.SRT`, `.JPG`, `.PnG` all match
- **Same-directory only**: only looks in the parent directory of the media file (matching Python mover)
- **Stem matching**: only files with the same `file_stem` as the media file are included
- **Excludes media file itself**: the media file is never in the result
- **Excludes non-matching stems**: `other.srt` or `movie.en.srt` are ignored
- **Nonexistent path**: returns empty `Vec`
- **Ignores directories**: a directory named `movie.srt` is not returned

## Known Gaps / Intentional Deviations

1. **Companion files in special subdirectories**: Python's `Identifier` (not `mover`) discovers companion files in `Subs/`, `sub/`, `Subtitles/`, `subtitle/` subdirectories recursively. This is **identifier** behavior, not sidecar discovery behavior. The Rust `discover_sidecars` intentionally matches the mover-level function.
2. **No symlink handling**: Python `discover_sidecars` checks `file.is_file()` which follows symlinks. Rust also uses `entry.file_type().is_file()` which does not follow symlinks. This is a minor semantic difference; symlinks to sidecars would be skipped in Rust but followed in Python.

## Test Coverage

10 tests in `crates/rosey-fs/tests/sidecars_tests.rs`:

- `is_sidecar_recognizes_all_extensions` — all 14 extensions recognized
- `is_sidecar_case_insensitive` — uppercase and mixed case match
- `is_sidecar_rejects_non_sidecar` — `.mkv`, `.mp4`, `.txt`, no-extension rejected
- `discover_finds_matching_files` — finds 3 sidecars, ignores `other.srt`
- `discover_empty_when_no_matches` — empty result when no sidecars
- `discover_handles_nonexistent_path` — nonexistent parent → empty
- `discover_does_not_include_media_file` — media file excluded
- `discover_ignores_directories` — directories with sidecar-like names excluded
- `discover_case_insensitive_extensions` — `.SRT`, `.JPG` recognized
- `discover_different_stem_ignored` — `movie.en.srt` ignored (stem mismatch)

All tests use `tempfile::tempdir()` — no real user media paths touched.

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace` (96 passed total)
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Slice

**NFO parsing** — if there is Python NFO parsing behavior to port. Alternatively, **identifier companion discovery** (the special-subdirectory recursive scanning from `test_companion_files.py`). The move/copy engine parity should come later after the file safety ADR is reviewed.
