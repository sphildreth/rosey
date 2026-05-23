# Rosey Agent Guardrails

Do not let agents broaden scope.

Hard rules:

- Do not mutate `../rosey` unless explicitly instructed.
- Do not implement more than one migration slice per task.
- Do not implement TUI screens before the core/CLI behavior is testable.
- Do not place design material in `/docs`; use `/design`.
- Do not add destructive file operations without dry-run tests and safety review.
- Do not claim parity unless tests or golden fixtures prove it.
- Do not hide failed commands.

Required final report:

```text
Agent used:
Files changed:
Behavior implemented:
Tests added/updated:
Commands run:
Known gaps:
Recommended next step:
```
