# ADR-0005: Use an operation journal for safe moves

Status: Accepted  
Date: 2026-05-23

## Context

Rosey performs potentially destructive operations on valuable media libraries.

Rollback inside a running process is not enough. If the process crashes, the computer reboots, or a network share disconnects mid-transfer, the user needs a durable record of what happened.

## Decision

Rosey will use a JSON Lines operation journal for move/copy/delete operations.

The journal records steps before and after important filesystem actions.

## Consequences

Positive:

- better crash recovery
- better support diagnostics
- safer destructive operations
- easier user trust

Negative:

- more implementation complexity
- journal files must be managed
- recovery behavior needs tests and docs

## Alternatives considered

- In-memory rollback only: rejected because it fails on crash.
- Log files only: rejected because logs are not structured enough for recovery.
