# Rosey Agent Instructions

Rosey is a media organizer for Jellyfin. This repository contains the Rust implementation.

## Repository Layout

```text
/docs       user-facing documentation only
/design     design docs, ADRs, prompts, planning notes
/design/adr architectural decision records
crates/     Rust workspace crates
tests/      cross-crate fixtures, golden files, integration assets
```

## Read First

Before changing code, read the smallest relevant set:

1. `design/SPEC.md`
2. `design/MIGRATION_PLAN.md`
3. `design/adr/`
4. the relevant `crates/*`
5. any legacy reference source only when explicitly needed for behavior investigation

## Non-Negotiables

- Do not mutate external reference repositories unless the user explicitly asks.
- Do not make broad rewrites. Work one focused behavior slice at a time.
- Do not implement destructive file operations without dry-run behavior, temp-dir tests, and a recovery/journal plan.
- Keep `/docs` user-facing and `/design` for design, ADRs, prompts, and planning notes.
- Add or update an ADR for architecture-significant changes.
- CLI and core engine behavior must be testable before TUI screens depend on it.
- Prefer explicit, typed domain models over stringly-typed behavior.
- Keep UI code out of `rosey-core`.
- Preserve documented behavior unless a change is intentional, tested, and documented.

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
