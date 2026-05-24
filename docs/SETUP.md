# Setup

Rosey Rust is currently a development-stage rewrite.

## Requirements

- Rust stable toolchain
- Cargo
- A terminal with ANSI support
- For Windows, Windows Terminal is recommended

## Build

```bash
cargo build --workspace
```

## Test

```bash
cargo test --workspace
```

## Run CLI

```bash
cargo run -p rosey-cli -- scan /path/to/source --json
cargo run -p rosey-cli -- identify "Example.Show.S01E02.mkv" --json
cargo run -p rosey-cli -- run /path/to/source --movies-target /movies --tv-target /tv
```

`scan` and `run` can use `paths.source` from `~/.config/rosey/rosey.json` when the source argument is omitted.

## Run TUI

```bash
cargo run -p rosey-tui -- /path/to/source /movies /tv
```

The TUI also reads `paths.source`, `paths.movies`, and `paths.tv` from config when those positional arguments are omitted.
