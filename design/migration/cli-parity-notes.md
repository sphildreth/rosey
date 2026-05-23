# CLI Parity Notes

## Status
Complete. Python CLI behavior from `rosey/cli.py` ported to `crates/rosey-cli/src/main.rs`.

## Ported Commands

| Python | Rust | Notes |
|--------|------|-------|
| `scan_directory` | `rosey scan <root>` | `--json`, `--max-workers` |
| `identify_file` | `rosey identify <path>` | `--json` |
| Full workflow | `rosey run <source>` | `--movies-target`, `--tv-target`, `--dry-run`/`--no-dry-run`, `--confidence`, `--conflict-policy`, `--json` |

## CLI Arguments

| Argument | Default | Description |
|---|---|---|
| `source` (positional) | required | Source directory to scan |
| `--movies-target` | `None` | Target directory for movies |
| `--tv-target` | `None` | Target directory for TV shows |
| `--dry-run` | `true` | Dry-run mode (default) |
| `--no-dry-run` | N/A | Disables dry-run, executes live moves |
| `--max-workers` | `8` | Concurrent workers for scanning |
| `--confidence` | `0` | Minimum confidence threshold (0-100) |
| `--conflict-policy` | `skip` | `skip`, `replace`, `keep_both` |
| `--json` | `false` | Output results as JSON |

## Workflow

1. **Scan** — `Scanner::scan()` finds video files
2. **Identify** — `identify_file()` parses each video's metadata using:
   - NFO parsing (`find_nfo_for_file` + `parse_nfo`)
   - Companion file discovery (`discover_companion_files`)
   - Filename pattern extraction (year, episode, date, part)
   - Folder context for season/year hints
3. **Score** — `score_identification()` computes confidence (0-100)
4. **Filter** — Results below `--confidence` are dropped
5. **Plan** — `plan_path()` computes Jellyfin-friendly destination
6. **Display** — Results grouped by confidence band (Green ≥70, Yellow 40-69, Red <40)
7. **Move** — If `--no-dry-run`, `move_with_sidecars()` executes the move

## Output Formats

- **Human-readable**: confidence label, title/year/episode, source path, destination, reasons
- **JSON**: structured `RunOutput` with green/yellow/red arrays, dry-run flag, move summary

## Error Handling

- Missing source directory → stderr + exit code 1
- No video files found → message + exit code 0
- Move errors → logged per-item, aggregate summary at end
- Never crashes mid-batch; completes all items and reports aggregate results

## Known Gaps / Intentional Deviations

1. **No config file support**: Python loads/saves config via `load_config()`/`save_config()`. Rust CLI does not implement config file persistence yet.
2. **No online metadata**: Python may call TMDB API for identification. Rust CLI is offline-only for now.
3. **No duration probing**: Python checks video duration for confidence scoring. Not implemented in Rust yet.
4. **Scoring is simplified**: Python `score_identification` is more nuanced (duration, folder structure, etc.). Rust uses a basic heuristic.
5. **No `--save-config` flag**: Not implemented.

## Test Coverage

CLI is tested indirectly via the underlying crate tests:
- `patterns_tests.rs` — 54 parser tests
- `planner_tests.rs` — 22 planner tests
- `scanner_tests.rs` — 9 scanner tests
- `sidecars_tests.rs` — 10 sidecar tests
- `companions_tests.rs` — 15 companion tests
- `nfo_tests.rs` — 17 NFO tests
- `mover_tests.rs` — 15 mover tests

Total: 146 tests across all crates.

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace` (146 passed total)
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Slice

**TUI implementation** — the CLI now has all the building blocks. The `rosey-tui` crate can be built on top of:
- `rosey-core` (parser, planner, models)
- `rosey-fs` (scanner, mover)
- `rosey-cli` (command structure, identification logic)

Alternatively, **metadata provider parity** (TMDB API client) can be implemented in `rosey-metadata` before the TUI, since online identification improves confidence scores significantly.
