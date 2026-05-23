# Documentation and ADR Rules

Use ADRs for architecture-significant decisions.

Create or update an ADR when changing:

- UI stack or major TUI architecture
- crate boundaries
- transfer/journal/recovery design
- metadata provider/cache architecture
- test strategy or parity strategy
- release/distribution approach
- destructive file operation semantics

ADR location:

```text
/design/adr/
```

ADR format:

```text
# ADR-NNNN: Title

Date:
Status:

## Context
## Decision
## Consequences
## Alternatives Considered
## Follow-up
```

Keep user-facing docs in `/docs`; keep design rationale in `/design`.
