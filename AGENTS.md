# Rosey Rust Agent Instructions

Rosey Rust is a Rust rewrite of the existing Python/PySide6 Rosey media organizer. Rosey organizes Movies and TV Shows into Jellyfin-friendly folders.

## Repository Layout

```text
/docs       user-facing documentation only
/design     design docs, ADRs, prompts, migration strategy
/design/adr architectural decision records
crates/     Rust workspace crates
tests/      cross-crate fixtures, golden files, integration assets
```

Expected sibling layout:

```text
../rosey        existing Python/PySide6 reference implementation
../rosey-rust   new Rust implementation
```

## Read First

Before changing code, read the smallest relevant set:

1. `design/SPEC.md`
2. `design/MIGRATION_STRATEGY.md`
3. `design/adr/`
4. the relevant `crates/*`
5. the matching Python source in `../rosey` when porting behavior

## Non-Negotiables

- Do not mutate `../rosey` unless the user explicitly asks. Treat it as the behavioral reference.
- Do not make broad rewrites. Work one migration slice at a time.
- Do not implement destructive file operations without dry-run behavior, temp-dir tests, and a recovery/journal plan.
- Keep `/docs` user-facing and `/design` for design, ADRs, prompts, and migration notes.
- Add or update an ADR for architecture-significant changes.
- CLI and core engine behavior must be testable before TUI screens depend on it.
- Prefer explicit, typed domain models over stringly-typed behavior.
- Keep UI code out of `rosey-core`.
- Preserve Python behavior during parity phases, even if a cleanup seems tempting. Record desired improvements separately.

## Required Quality Gates

Run before finishing:

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

When changing docs or prompts, also check links and file locations.

## Crate Boundaries

```text
rosey-core       domain models, parsing, planner, scoring; no UI
rosey-fs         scanning, file ops, sidecars, transfer engine
rosey-metadata   provider clients, cache, metadata adapters
rosey-cli        headless command interface and parity harness
rosey-tui        Ratatui/Crossterm UI only
```

Dependency direction:

```text
rosey-fs       -> rosey-core
rosey-metadata -> rosey-core
rosey-cli      -> rosey-core, rosey-fs, rosey-metadata
rosey-tui      -> rosey-core, rosey-fs, rosey-metadata
```

No circular dependencies.

## Testing Strategy

Use the old Python repo to create behavior parity where possible.

Recommended Rust testing tools:

```text
insta              snapshot/golden tests
proptest           property-based parser/path tests
tempfile           filesystem integration tests
assert_cmd         CLI tests
pretty_assertions  readable diffs
```

## Output Expectations

Every agent response should include:

```text
Files changed
Behavior implemented
Tests added/updated
Commands run
Known gaps
Recommended next step
```

If a quality gate cannot be run, say so plainly and explain why.
