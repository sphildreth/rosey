---
name: rosey-file-engine-safety
description: Use when implementing scanning, sidecar discovery, transfer planning, copy/move execution, rollback, operation journal, or recovery behavior.
---


# Rosey File Engine Safety Skill

Use this skill for scanner, sidecar, mover, transfer, journal, and recovery work.

## Rust Target

```text
crates/rosey-fs/
```

## Required Safety Model

All destructive file behavior must have:

- dry-run mode
- explicit conflict policy
- temp-dir integration tests
- cancellation-safe state transitions where practical
- operation events for CLI/TUI
- structured errors
- source deletion only after destination verification
- rollback or recovery story

## Transfer Rules

Same volume:

```text
rename/replace path may be used when safe
```

Cross volume:

```text
copy -> verify -> delete source
```

Never:

```text
delete source before verification
hide partial copy failures
test with real user media paths
```

## Event Model

Prefer event emission over direct UI coupling:

```rust
TransferEvent::Started
TransferEvent::Progress
TransferEvent::Verified
TransferEvent::Completed
TransferEvent::Failed
TransferEvent::RolledBack
```

## Documentation

Update or create ADRs for:

- operation journal design
- cross-volume verification rules
- rollback/recovery semantics
