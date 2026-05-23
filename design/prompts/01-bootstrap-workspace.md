# Prompt: Bootstrap Workspace

Create or refine the initial Rust workspace for Rosey Rust.

## Goal

Build the structural foundation only.

## Required output

- Cargo workspace
- crates:
  - `rosey-core`
  - `rosey-fs`
  - `rosey-metadata`
  - `rosey-cli`
  - `rosey-tui`
- `/docs`
- `/design`
- `/design/adr`
- `/design/prompts`
- `AGENTS.md`
- GitHub Actions Rust workflow
- basic README

## Do not

- Do not port the whole Python repo.
- Do not implement real destructive move/copy behavior.
- Do not add a web server.
- Do not add a GUI framework.

## Success criteria

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
