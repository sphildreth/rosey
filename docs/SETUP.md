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
```

## Run TUI

```bash
cargo run -p rosey-tui
```

The TUI is a starter placeholder until the core and CLI parity harness are implemented.
