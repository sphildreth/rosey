# ADR-0002: Use a separate rosey-rust repository during migration

Status: Accepted  
Date: 2026-05-23

## Context

The existing Python repo contains PySide6 UI code, Python packaging, pytest configuration, PyInstaller scripts, and Python-specific documentation.

Converting that repo in place would create a confusing hybrid repository and increase the risk of coding agents modifying unrelated areas.

## Decision

Create a separate `rosey-rust` repository.

Keep the Python repo in place as the behavioral reference until the Rust rewrite is satisfactory. After the Rust version is ready, archive or rename the old Python repository and rename `rosey-rust` to `rosey`.

## Consequences

Positive:

- clean Rust repo from day one
- easier agent instructions
- no half-migrated source tree
- existing Python app remains available
- final cutover is clean

Negative:

- issues/releases/history may need careful migration
- links and badges must be updated during cutover
- duplicated docs may exist temporarily

## Alternatives considered

- In-place rewrite: rejected because it creates a messy mixed repo.
- Monorepo with Python and Rust: rejected because the Python code is a temporary reference, not a long-term component.
