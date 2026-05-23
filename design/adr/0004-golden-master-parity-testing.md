# ADR-0004: Use golden-master parity testing against Python Rosey

Status: Accepted  
Date: 2026-05-23

## Context

The existing Python implementation has useful behavior, tests, and edge cases. Starting from scratch risks losing this behavior.

## Decision

Use golden-master parity tests.

The Python implementation will produce normalized JSON outputs for selected fixtures. The Rust implementation must match those outputs unless an intentional behavior change is documented.

## Consequences

Positive:

- preserves known-good behavior
- gives agents objective success criteria
- enables module-by-module migration
- reduces rewrite risk

Negative:

- golden files require maintenance
- not all behavior should be matched exactly
- intentional differences must be documented

## Alternatives considered

- Manual rewrite from docs: rejected as too error-prone.
- Direct line-by-line port: rejected because Rust should have its own clean architecture.
