# Architecture Decision Records

ADRs capture significant decisions for Rosey Rust.

## Create a new ADR

```bash
./scripts/new-adr.sh "Use operation journal for destructive file operations"
```

## Index

- [ADR-0001: Rewrite Rosey in Rust with Ratatui/Crossterm TUI](./0001-rust-ratatui-rewrite.md)
- [ADR-0002: Use a separate rosey-rust repository during migration](./0002-separate-rosey-rust-repository.md)
- [ADR-0003: Split core, filesystem, CLI, metadata, and TUI crates](./0003-split-core-filesystem-cli-metadata-and-tui-crates.md)
- [ADR-0004: Use golden-master parity testing against Python Rosey](./0004-golden-master-parity-testing.md)
- [ADR-0005: Use an operation journal for safe moves](./0005-operation-journal-for-safe-moves.md)
- [ADR-0006: Terminal-native parity and safety decisions](./0006-terminal-native-parity-and-safety-decisions.md)
