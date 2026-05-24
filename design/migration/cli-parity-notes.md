# CLI Parity Notes

## Status
Partial. Python CLI behavior from `rosey/cli.py` is ported to `crates/rosey-cli/src/main.rs` for scan, identify, and run workflows, with config-driven startup defaults wired in for supported fields.

## Ported Commands

| Python | Rust | Notes |
|--------|------|-------|
| `scan_directory` | `rosey scan <root>` | `--json`, `--max-workers` |
| `identify_file` | `rosey identify <path>` | `--json` |
| Full workflow | `rosey run <source>` | `--movies-target`, `--tv-target`, `--dry-run`/`--no-dry-run`, `--confidence`, `--conflict-policy`, `--json` |

## CLI Arguments

| Argument | Default | Description |
|---|---|---|
| `source` (positional) | `paths.source` | Source directory to scan |
| `--movies-target` | `paths.movies` | Target directory for movies |
| `--tv-target` | `paths.tv` | Target directory for TV shows |
| `--dry-run` | `true` | Dry-run mode (default) |
| `--no-dry-run` | N/A | Disables dry-run, executes live moves |
| `--max-workers` | `scanning.concurrency_local` | Concurrent workers for scanning |
| `--confidence` | `0` | Minimum confidence threshold (0-100) |
| `--conflict-policy` | `behavior.conflict_policy` | `skip`, `replace`, `keep_both`; `ask` maps to safe skip |
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
6. **Display** — Results grouped by configured confidence bands
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

1. **Config support is partial**: Rust now loads config defaults at startup for supported CLI/TUI fields, but it does not yet implement Python's full config surface or `--save-config` behavior.
2. **No online metadata**: Python may call TMDB API for identification. Rust CLI is offline-only for now.
3. **No duration probing**: Python checks video duration for confidence scoring. Not implemented in Rust yet.
4. **Scoring is simplified**: Python `score_identification` is more nuanced (duration, folder structure, etc.). Rust uses a basic heuristic.
5. **Dry-run config is intentionally not destructive in CLI**: Python's CLI defaults to dry-run despite loading config. Rust follows that safety behavior; live moves require explicit `--no-dry-run`.
6. **No `--save-config` flag**: Not implemented.

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

Wire metadata providers and duration-aware scoring into the shared identification flow so the CLI and TUI produce confidence scores closer to Python Rosey.
