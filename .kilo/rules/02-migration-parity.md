# Migration Parity Rules

Rosey Rust is a rewrite of the Python/PySide6 Rosey project.

When porting behavior:

1. Read the matching Python source and tests from `../rosey`.
2. Implement the smallest Rust slice possible.
3. Add Rust unit, integration, snapshot, or golden tests.
4. Preserve current Python behavior unless the user explicitly asks for a behavior change.
5. Record known gaps in `/design/migration/`.
6. Do not delete or mutate Python reference files.

When behavior differs intentionally, document:

```text
Python behavior
Rust behavior
Reason for deviation
Test coverage
ADR or design note link
```
