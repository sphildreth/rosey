# Rosey Model Routing Rules

Use the specialized Rosey agents instead of one general agent for everything.

Recommended routing:

- `rosey-orchestrator`: plan a slice, delegate, enforce boundaries.
- `rosey-phase-executor`: implement one migration slice at a time.
- `rosey-rust-fixer`: fix compile, test, fmt, and clippy failures.
- `rosey-bulk-worker`: generate bulk tests, fixtures, first-pass mechanical code, or docs.
- `rosey-reviewer`: review correctness, architecture, tests, and parity.
- `rosey-file-safety-reviewer`: review scanner/mover/copy/delete/journal safety.
- `rosey-test-runner`: run and triage cargo gates.
- `rosey-docs-adr`: create/update `/design`, ADRs, and agent prompts.

Model recommendations:

- Kimi K2.6/K2.5/K2 Thinking for orchestration and full migration slices.
- Qwen3-Coder-Next/Qwen3-Coder for focused Rust implementation and fixes.
- DeepSeek V4-Pro for cheap bulk implementation, fixture generation, and documentation.
- GLM-5.1/GLM-5 for review, critique, and safety analysis.
- MiniMax M2.7 only as an experimental second-opinion agent.
