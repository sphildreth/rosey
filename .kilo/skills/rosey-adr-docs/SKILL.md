---
name: rosey-adr-docs
description: Use when creating or updating Rosey design documents, ADRs, migration notes, or coding-agent prompts.
---


# Rosey ADR and Design Docs Skill

Use this skill for `/design`, `/design/adr`, and prompt work.

## Directory Rules

```text
/docs       user-facing documentation
/design     PRD, SPEC, ADRs, prompts, migration notes
```

## ADR Triggers

Create or update ADRs for architecture-significant changes:

- crate boundaries
- TUI architecture
- operation journal
- destructive file operation semantics
- provider/cache strategy
- testing/parity strategy
- packaging/release strategy

## ADR Template

```markdown
# ADR-NNNN: Title

Date:
Status:

## Context

## Decision

## Consequences

## Alternatives Considered

## Follow-up
```

## Prompt Rules

Prompts go under:

```text
/design/prompts/
```

Prompts should include:

```text
Role
Context
Objective
Source files
Required work
Non-goals
Quality gates
Deliverables
```
