# Migration Plan — Python Rosey to Rust Rosey

## Strategy

Use a separate `rosey-rust` repo while keeping the existing Python repo intact as the behavioral reference.

Expected layout:

```text
../rosey        Python/PySide6 Rosey
../rosey-rust   Rust rewrite
```

## Why separate repos

- avoids a half-Python/half-Rust transitional mess
- lets agents work in a clean Rust workspace
- keeps the existing app usable until parity is proven
- makes final cutover simple: archive old repo, rename new repo

## Migration sequence

### Phase 0 — Preserve Python behavior [COMPLETE]

- [x] leave Python repo working
- [x] tag a final Python baseline when ready (tag: `python-baseline`)
- [x] add golden exporter scripts only if useful (`scripts/golden_exporter.py`)
- [x] collect fixtures for important edge cases (`tests/fixtures/`: media_tree_001, media_tree_sidecars, media_tree_conflicts, nfo, filenames)

### Phase 1 — Rust workspace [COMPLETE]

- [x] create workspace
- [x] create crates
- [x] add AGENTS.md
- [x] add PRD/SPEC/ADRs
- [x] add CI

### Phase 2 — Core models [COMPLETE]

- [x] port Pydantic model shapes into typed Rust structs
- [x] serialize/deserialize to JSON
- [x] ensure model JSON can be compared to Python output

### Phase 3 — Parser parity [COMPLETE]

- [x] port filename pattern logic
- [x] port year/date/episode/part extraction
- [x] port title cleanup
- [x] add parity/golden tests

### Phase 4 — Planner parity [COMPLETE]

- [x] port Jellyfin destination path logic
- [x] port sanitization rules
- [x] port conflict suffix behavior
- [x] add snapshot tests

### Phase 5 — Scanner parity [COMPLETE]

- [x] port extension filtering
- [x] port symlink behavior
- [x] port error behavior
- [x] compare scan outputs on fixture trees

### Phase 6 — Mover safety [COMPLETE]

- [x] implement dry-run first
- [x] add preflight checks
- [x] add operation journal (see ADR-0005; `crates/rosey-fs/src/journal.rs`)
- [x] implement same-volume move
- [x] implement cross-volume copy+verify+delete
- [x] implement sidecar moves
- [x] implement rollback/recovery tests

### Phase 7 — CLI [COMPLETE]

- [x] expose scan/identify/plan/move
- [x] make JSON output stable
- [x] use CLI for parity testing
- [x] read supported config defaults for paths, scanning, conflict policy, and confidence bands
- [x] implement Python `--save-config`
- [x] integrate provider-confirmed path TMDB IDs and duration-aware identification into CLI identification

### Phase 8 — TUI [COMPLETE]

- [x] build Ratatui app state (`crates/rosey-tui/src/app.rs` — Screen, AppState, IdentifiedItem, TransferItem, sorting)
- [x] render dashboards and result tables (7 screens: Dashboard, Scan Results, Plan Preview, Transfer Queue, Logs/Recovery, Settings, Help)
- [x] wire engine event stream (scan/identify/plan/move driven by keyboard with state machine)
- [x] require confirmation before destructive execution (confirmation dialog with [y]/[n] for live mode)
- [x] read supported config defaults at startup
- [x] keep scan/plan/move work off the input/render loop and show progress feedback
- [x] persist current in-memory TUI settings snapshot to `rosey.json`
- [x] implement terminal settings editing for Python config fields
- [x] add manual identify overlay with provider search when configured
- [x] expose journal inspection and explicit post-move cleanup commands
- [x] add terminal-level smoke tests for key screens

### Phase 9 — Metadata [COMPLETE]

- [x] port provider interfaces
- [x] port cache
- [x] add TMDB/TVDB support
- [x] keep offline identification fully functional without providers
- [x] wire provider-confirmed path TMDB IDs into CLI/TUI identification flow
- [x] add cache-backed parity tests for provider-enabled scoring behavior without network access
- [x] document optional real-API smoke as release validation

### Phase 10 — Cutover [NOT STARTED]

- [ ] freeze Python repo
- [ ] tag final Python version
- [ ] rename old `rosey` to `rosey-python-archive`
- [ ] rename `rosey-rust` to `rosey`
- [ ] update README, badges, links, releases

## Agent rule

Do not ask an agent to “convert the repo.”

Ask the agent to port exactly one module or behavior group at a time.
