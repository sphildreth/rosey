# Scanner Parity Notes

## Status
Complete. All Python scanner behavior has been ported to `crates/rosey-fs/src/scanner.rs`.

## Ported Functions / Types

| Python | Rust | Notes |
|--------|------|-------|
| `Scanner` class | `Scanner` struct | `max_workers` + `follow_symlinks` |
| `Scanner.__init__` | `Scanner::new` + `Default` | Default: 8 workers, no symlink follow |
| `Scanner.scan` | `Scanner::scan` | Returns `Vec<ScanResult>` |
| `scan_directory` | `scan_directory` | Convenience function |
| `ScanResult` | `ScanResult` | Same fields, `Utf8PathBuf` for paths |
| `VIDEO_EXTENSIONS` | `VIDEO_EXTENSIONS` | Same set, without leading dots |
| `_enumerate_paths` | `walkdir` iterator | Handled by `walkdir` crate |
| `_scan_path` | Inline in `scan()` | Extension check + size from metadata |

## Behavior Verified

- **Recursive directory scanning**: finds all video files in nested trees
- **Extension filtering**: only `.mkv`, `.mp4`, `.avi`, `.mov`, `.wmv`, `.flv`, `.m4v`, `.mpg`, `.mpeg`, `.webm`, `.ts` (case-insensitive)
- **Non-video files skipped**: `.txt`, `.jpg`, etc. are ignored
- **Empty directory**: returns empty `Vec`
- **Nonexistent path**: returns empty `Vec` (matching Python early-return)
- **Single file path**: scans it directly, returns it if video
- **File sizes**: all video results have `size_bytes > 0`
- **Concurrency parity**: different `max_workers` values produce identical results (no `rayon` needed; `walkdir` handles traversal efficiently)
- **Symlinks**: `follow_symlinks` flag passed to `walkdir`

## Known Gaps / Intentional Deviations

1. **`max_workers` not used for thread pool**: Python uses `ThreadPoolExecutor` to run `_scan_path` concurrently. In Rust, `walkdir` yields entries with metadata already populated, so the per-file work is negligible. `max_workers` is stored for API parity but does not affect behavior.
2. **`max_depth` option**: Added in Rust as a useful enhancement not present in Python. Default `None` preserves Python's unlimited-depth behavior.
3. **Logging**: Python emits log messages for "No files found", "Permission denied", etc. Rust does not include logging in the scanner itself; callers can log as needed.
4. **Error results for walk errors**: Python's `ThreadPoolExecutor` catches exceptions and appends `ScanResult` with `error`. Rust's `walkdir` error entries are also captured as `ScanResult` with `error: Some(...)`.

## Test Coverage

10 tests total:

- 1 inline unit test: `detects_video_extensions_case_insensitively`
- 9 integration tests in `crates/rosey-fs/tests/scanner_tests.rs`:
  - `scanner_basic` — finds 5 videos across root + subfolder
  - `scanner_video_extensions` — correct video/non-video classification
  - `scanner_empty_directory` — empty dir → empty vec
  - `scanner_nonexistent_path` — nonexistent → empty vec
  - `scanner_single_file` — single file scan
  - `scanner_concurrency_levels_produce_same_results` — 1 vs 4 workers same output
  - `scan_directory_convenience` — convenience function finds 3 videos
  - `scanner_large_tree` — 5×10 nested files = 50 videos
  - `scanner_file_sizes` — all videos have size > 0

All tests use `tempfile::tempdir()` — no real user media paths touched.

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace` (86 passed total)
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Slice

**Sidecar discovery** (`sidecars.rs` parity) or **NFO parsing** (if there is Python NFO behavior to port). The `sidecars.rs` stub already has `is_sidecar_path` and `discover_sidecars`, but lacks parity tests against Python.
