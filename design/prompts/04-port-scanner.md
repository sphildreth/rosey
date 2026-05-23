# Prompt: Port Scanner

Port scanner behavior into `rosey-fs`.

## Python source to inspect

Look in `../rosey` for scanner modules and scanner tests.

## Rust target

- `crates/rosey-fs/src/scanner.rs`

## Requirements

Support:

- recursive scanning
- video extension detection
- symlink option
- permission errors without crashing
- deterministic test output
- JSON output through `rosey-cli scan`

## Success criteria

```bash
cargo run -p rosey-cli -- scan tests/fixtures/tree_001 --json
cargo test -p rosey-fs
```
