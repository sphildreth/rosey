set shell := ["bash", "-uc"]

default:
    just check

fmt:
    cargo fmt --all

test:
    cargo test --workspace

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

check: fmt clippy test

scan-example:
    cargo run -p rosey-cli -- scan . --json

identify-example:
    cargo run -p rosey-cli -- identify "Example.Show.S01E02.mkv" --json

tui:
    cargo run -p rosey-tui
