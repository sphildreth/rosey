# CLI Parity Notes

## Status
Complete. Python CLI behavior from `rosey/cli.py` is ported to `crates/rosey-cli/src/main.rs` for scan, identify, and run workflows, with config-driven startup defaults, metadata-aware identification, and duration-aware core scoring wired in.

## Ported Commands

| Python | Rust | Notes |
|--------|------|-------|
| `scan_directory` | `rosey scan <root>` | `--json`, `--max-workers` |
| `identify_file` | `rosey identify <path>` | `--json` |
| Full workflow | `rosey run <source>` | `--movies-target`, `--tv-target`, `--dry-run`/`--no-dry-run`, `--confidence`, `--conflict-policy`, `--json`, `--save-config` |

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
2. **Identify** — `identify_file_with_metadata()` parses each video's metadata using:
   - NFO parsing (`find_nfo_for_file` + `parse_nfo`)
   - Companion file discovery (`discover_companion_files`)
   - Filename pattern extraction (year, episode, date, part)
   - Folder context for season/year hints
   - TMDB enrichment for path-embedded `[tmdbid-*]` values only after provider/cache confirmation when online providers are enabled and a TMDB API key is configured
3. **Score** — `score_identification_result()` computes confidence (0-100) from the identified item using the Python scoring weights
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

## Intentional Deviations

1. **Config support is terminal-aware**: Rust preserves Python config sections, including `ui`, but the CLI intentionally ignores PySide6 window geometry/splitter values.
2. **Metadata lookup matches identifier behavior**: automatic identification enriches files that already carry a `[tmdbid-*]` tag in the path; broader title search is exposed through the TUI identify overlay rather than the batch CLI.
3. **Dry-run config is intentionally not destructive in CLI**: Python's CLI defaults to dry-run despite loading config. Rust follows that safety behavior; live moves require explicit `--no-dry-run`.
4. **`--save-config` is limited to path persistence**: The Rust CLI saves CLI-provided `source`, `movies_target`, and `tv_target` values into `rosey.json`. It does not persist broader config edits from the CLI.

## Test Coverage

CLI is tested indirectly via the underlying crate tests:
- `patterns_tests.rs` — 54 parser tests
- `planner_tests.rs` — 22 planner tests
- `scanner_tests.rs` — 9 scanner tests
- `sidecars_tests.rs` — 11 sidecar tests
- `companions_tests.rs` — 16 companion tests
- `nfo_tests.rs` — 17 NFO tests
- `mover_tests.rs` — 17 mover tests

The workspace also includes config, identifier, scorer, metadata-cache, and provider-confirmation tests.

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Step

Add optional real-provider smoke tests gated on user-provided API keys if API behavior needs to be verified before release.
