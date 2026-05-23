# Rosey Rust Kilo Agent Usage Examples

## Parser parity

```text
Use rosey-orchestrator. Implement parser parity as the next migration slice. Use the Python repo at ../rosey as the read-only reference. Delegate implementation to rosey-phase-executor, cargo fixes to rosey-rust-fixer, and review to rosey-reviewer.
```

## Planner parity

```text
Use rosey-phase-executor. Implement Jellyfin planner parity only. Do not implement scanning, moving, metadata, or TUI screens. Add tests and update /design/migration/planner-parity-notes.md.
```

## Fix a broken build

```text
Use rosey-rust-fixer. Fix only the cargo fmt/test/clippy failures. Do not broaden the feature scope.
```

## Generate more test cases

```text
Use rosey-bulk-worker. Generate table-driven Rust tests for parser edge cases from ../rosey tests. Keep fixtures tiny and deterministic.
```

## Review a PR/branch

```text
Use rosey-reviewer. Review the current changes for crate boundaries, migration parity, tests, docs, and quality gates. Do not edit files.
```

## Review mover safety

```text
Use rosey-file-safety-reviewer. Review copy/move/delete/journal behavior for destructive-operation safety. Do not edit files.
```

## Create an ADR

```text
Use rosey-docs-adr. Create an ADR in /design/adr for the operation journal design. Keep /docs user-facing only.
```

## Get an alternate opinion

```text
Use rosey-minimax-second-opinion. Provide a second opinion on the current implementation risks and missing tests. Do not edit files.
```
