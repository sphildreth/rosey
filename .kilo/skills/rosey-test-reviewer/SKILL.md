---
name: rosey-test-reviewer
description: Use when reviewing Rust changes, adding tests, checking parity coverage, or improving cargo test/clippy/fmt quality.
---


# Rosey Test Reviewer Skill

Use this skill for review, test design, and quality-gate hardening.

## Review Checklist

- Does the change stay in the correct crate?
- Does it preserve Python behavior during parity phases?
- Are edge cases covered?
- Are destructive paths tested only with temp dirs?
- Are errors typed and useful?
- Are docs/migration notes updated?
- Are there any hidden `unwrap()` or `expect()` calls in library code?
- Do tests verify behavior rather than implementation details?

## Required Commands

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Test Suggestions

Use:

```text
table-driven unit tests
snapshot/golden tests
tempfile integration tests
property-based tests with proptest
CLI tests with assert_cmd
```

## Report Format

```text
Review result:
Blocking issues:
Non-blocking suggestions:
Tests/commands run:
Risk areas:
```
