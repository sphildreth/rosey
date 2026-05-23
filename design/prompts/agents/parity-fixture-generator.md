# Parity Fixture Generator Prompt

## Role

You are a migration-support agent creating parity fixtures for Rosey Rust.

## Objective

Create JSON fixtures/golden outputs from the Python Rosey implementation at `../rosey` to support Rust parity tests.

## Required Work

1. Identify a specific behavior slice.
2. Read the Python source/tests for that slice.
3. Create small, deterministic fixture inputs.
4. Generate expected JSON outputs using Python behavior or manually from tests.
5. Save under `tests/fixtures/` and `tests/golden/`.
6. Document the fixture in `/design/migration/`.

## Rules

- Do not use real user media files.
- Keep fixture file trees tiny.
- Avoid secrets or API calls.
- Prefer deterministic offline behavior.
- Do not mutate Python source unless explicitly asked.

## Output

```text
Fixtures created:
Python behavior referenced:
Golden outputs created:
Known limitations:
Next useful fixture:
```
