# ADR-0001: Rewrite Rosey in Rust with Ratatui/Crossterm TUI

Status: Accepted  
Date: 2026-05-23

## Context

The existing Rosey application is written in Python with Qt6 bindings. It has useful behavior and tests, but Python environments, virtual environments, packaging, and runtime performance have become undesirable for the long-term direction.

Rosey is fundamentally a filesystem-heavy organizer for Jellyfin users. It benefits from a fast, safe core, straightforward binary distribution, and a power-user interface.

## Decision

Rosey v2 will be implemented in Rust.

The primary interactive UI will be a terminal UI using Ratatui and Crossterm. A CLI will be built before the TUI so the engine can be tested, scripted, and compared against Python behavior.

## Consequences

Positive:

- no Python venv
- native binary distribution
- strong type safety
- strong filesystem and concurrency control
- good fit for coding-agent module-by-module work
- strong fit for Jellyfin/self-hosting users

Negative:

- terminal UI may be less approachable for casual desktop users
- Rust learning curve is higher than Python
- a future GUI may still be needed if broader desktop adoption matters

## Alternatives considered

- Python/PySide6: rejected due to packaging/runtime friction.
- .NET/Avalonia: strong option, but less distinctive and less Rust-core aligned.
- Rust/Tauri: strong option, but introduces web frontend stack.
- Rust/Iced: strong pure-Rust GUI option, but TUI better matches the power-user direction for v2.
