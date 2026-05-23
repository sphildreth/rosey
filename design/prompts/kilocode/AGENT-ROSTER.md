# Rosey Rust Kilo Agent Roster

## Primary agents

### rosey-orchestrator

Use for high-level migration coordination. Recommended model: Kimi K2.6/K2.5/K2 Thinking.

### rosey-phase-executor

Use for one scoped implementation slice. Recommended model: Kimi K2.6/K2.5.

### rosey-docs-adr

Use for ADRs, design docs, migration notes, and prompts. Recommended model: DeepSeek/Kimi/GLM.

## Subagents

### rosey-rust-fixer

Compile/test/clippy fixer. Recommended model: Qwen3-Coder-Next/Qwen3-Coder.

### rosey-bulk-worker

Bulk tests, fixtures, mechanical conversions, docs drafts. Recommended model: DeepSeek V4-Pro.

### rosey-reviewer

Read-mostly reviewer. Recommended model: GLM-5.1/GLM-5.

### rosey-file-safety-reviewer

Destructive file operation reviewer. Recommended model: GLM-5.1/GLM-5/Kimi.

### rosey-test-runner

Runs and triages cargo gates. Recommended model: Qwen3-Coder-Next/DeepSeek.

### rosey-minimax-second-opinion

Experimental read-only second opinion. Recommended model: MiniMax M2.7.
