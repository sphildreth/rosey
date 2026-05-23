# Prompt: Port Safe Mover

Port safe move/copy behavior into `rosey-fs`.

## Python source to inspect

Look in `../rosey` for mover modules and mover tests.

## Rust target

- `crates/rosey-fs/src/mover.rs`
- `crates/rosey-fs/src/journal.rs`
- `crates/rosey-fs/src/sidecars.rs`

## Requirements

Implement in this order:

1. dry-run only
2. preflight checks
3. sidecar discovery
4. conflict suffixes
5. same-volume rename
6. cross-volume copy + verify
7. source delete only after verify
8. JSON Lines journal
9. rollback/recovery

## Safety requirements

- Use temp-dir tests.
- No destructive default behavior.
- CLI execute mode must be explicit.
- TUI execute mode must require confirmation.

## Success criteria

- tests cover skip/replace/keep-both
- tests cover sidecars
- tests cover rollback
- tests cover journal output
