# Prompt: Port Filename Patterns

Port filename/folder parsing behavior from the Python Rosey repo into `rosey-core`.

## Python source to inspect

Look in `../rosey` for pattern/identifier modules and tests related to:

- year extraction
- daily date extraction
- episode extraction
- episode ranges
- title cleanup
- season folder detection
- multipart/part extraction

## Rust target

- `crates/rosey-core/src/patterns.rs`
- tests under `crates/rosey-core`

## Requirements

- Preserve Python behavior where sensible.
- Add direct Rust unit tests.
- Add golden tests if Python fixtures already exist.
- Document intentional differences.

## Success criteria

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p rosey-core
```
