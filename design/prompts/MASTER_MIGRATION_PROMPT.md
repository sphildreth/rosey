# Master Migration Prompt

You are working in `rosey-rust`, a Rust rewrite of the Python/PySide6 Rosey media organizer.

The existing Python repository is expected at `../rosey`.

Your job is not to invent a new product. Your job is to port useful behavior from Python Rosey into a clean Rust architecture.

## Hard requirements

- Keep `/docs` for user-facing documents.
- Keep `/design` for PRD, SPEC, ADRs, prompts, and migration documents.
- Add ADRs for meaningful architecture decisions.
- Migrate one behavior group at a time.
- Add tests for every migrated behavior.
- Prefer golden parity against Python outputs when behavior already exists.
- Keep the TUI thin; business logic belongs in core crates.
- Do not implement destructive file operations without dry-run, temp-dir tests, and journal strategy.

## Before coding

Read:

- `AGENTS.md`
- `design/PRD.md`
- `design/SPEC.md`
- `design/MIGRATION_PLAN.md`
- `design/PARITY_STRATEGY.md`

Then inspect only the relevant Python files in `../rosey`.

## Done criteria

Run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Update docs and ADRs when decisions change.
