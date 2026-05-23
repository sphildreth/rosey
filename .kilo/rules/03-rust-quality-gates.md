# Rust Quality Gates

Before completing any coding task, run:

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If formatting changes are needed, run:

```bash
cargo fmt --all
```

Code conventions:

- Prefer `Result<T, RoseyError>` for recoverable core/domain errors.
- Use `thiserror` for typed errors.
- Use `anyhow` only at binary/application boundaries when helpful.
- Prefer `camino::Utf8PathBuf` for app-level paths that must serialize or render cleanly.
- Avoid `unwrap()` and `expect()` in library code except for compile-time/static invariants.
- Keep public APIs documented when they cross crate boundaries.
