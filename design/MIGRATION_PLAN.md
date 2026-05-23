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

### Phase 0 — Preserve Python behavior

- leave Python repo working
- tag a final Python baseline when ready
- add golden exporter scripts only if useful
- collect fixtures for important edge cases

### Phase 1 — Rust workspace

- create workspace
- create crates
- add AGENTS.md
- add PRD/SPEC/ADRs
- add CI

### Phase 2 — Core models

- port Pydantic model shapes into typed Rust structs
- serialize/deserialize to JSON
- ensure model JSON can be compared to Python output

### Phase 3 — Parser parity

- port filename pattern logic
- port year/date/episode/part extraction
- port title cleanup
- add parity/golden tests

### Phase 4 — Planner parity

- port Jellyfin destination path logic
- port sanitization rules
- port conflict suffix behavior
- add snapshot tests

### Phase 5 — Scanner parity

- port extension filtering
- port symlink behavior
- port error behavior
- compare scan outputs on fixture trees

### Phase 6 — Mover safety

- implement dry-run first
- add preflight checks
- add operation journal
- implement same-volume move
- implement cross-volume copy+verify+delete
- implement sidecar moves
- implement rollback/recovery tests

### Phase 7 — CLI

- expose scan/identify/plan/move
- make JSON output stable
- use CLI for parity testing

### Phase 8 — TUI

- build Ratatui app state
- render dashboards and result tables
- wire engine event stream
- require confirmation before destructive execution

### Phase 9 — Metadata

- port provider interfaces
- port cache
- add TMDB/TVDB support
- keep offline identification fully functional without providers

### Phase 10 — Cutover

- freeze Python repo
- tag final Python version
- rename old `rosey` to `rosey-python-archive`
- rename `rosey-rust` to `rosey`
- update README, badges, links, releases

## Agent rule

Do not ask an agent to “convert the repo.”

Ask the agent to port exactly one module or behavior group at a time.
