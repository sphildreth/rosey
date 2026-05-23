# ADR-0003: Split core, filesystem, CLI, metadata, and TUI crates

Status: Accepted  
Date: 2026-05-23

## Context

Rosey has separable concerns:

- domain models and parsing
- filesystem scanning and moves
- provider metadata
- command-line automation
- terminal UI

Keeping these concerns separated improves testability and keeps the UI from becoming the engine.

## Decision

Use a Cargo workspace with these crates:

- `rosey-core`
- `rosey-fs`
- `rosey-metadata`
- `rosey-cli`
- `rosey-tui`

## Consequences

Positive:

- cleaner boundaries
- easier testing
- easier agent work
- CLI and TUI share the same engine
- future GUI can reuse core crates

Negative:

- slightly more project setup
- APIs between crates must be maintained thoughtfully

## Alternatives considered

- Single binary crate: rejected because it would blur UI and engine logic.
- Core + one app crate only: rejected because CLI and TUI have different responsibilities.
