# KiloCode Model Pinning Notes for Rosey Rust Agents

Kilo agents can pin a model in YAML frontmatter using:

```yaml
model: provider/model-id
```

The exact provider/model string depends on your Kilo Gateway, OpenRouter, Ollama Cloud, local Ollama, or other configured provider.

The active agent files in `.kilo/agents/` intentionally do **not** pin exact models by default so they remain usable immediately.

## Recommended model mapping

| Agent | Recommended model |
|---|---|
| `rosey-orchestrator` | Kimi K2.6, Kimi K2.5, or Kimi K2 Thinking |
| `rosey-phase-executor` | Kimi K2.6 or Kimi K2.5 |
| `rosey-rust-fixer` | Qwen3-Coder-Next or Qwen3-Coder |
| `rosey-bulk-worker` | DeepSeek V4-Pro |
| `rosey-reviewer` | GLM-5.1 or GLM-5 |
| `rosey-file-safety-reviewer` | GLM-5.1, GLM-5, or Kimi |
| `rosey-test-runner` | Qwen3-Coder-Next or DeepSeek V4-Pro |
| `rosey-docs-adr` | DeepSeek V4-Pro, Kimi, or GLM |
| `rosey-minimax-second-opinion` | MiniMax M2.7 |

## Example frontmatter after you know exact IDs

These are examples only. Confirm the exact provider/model strings in your Kilo environment.

```yaml
---
description: Implements one Rosey Rust migration slice.
mode: all
model: openrouter/moonshotai/kimi-k2.5
temperature: 0.15
---
```

```yaml
---
description: Focused Rust fixer.
mode: subagent
model: openrouter/qwen/qwen3-coder
temperature: 0.05
---
```

```yaml
---
description: Bulk worker.
mode: subagent
model: openrouter/deepseek/deepseek-v4-pro
temperature: 0.2
---
```

```yaml
---
description: Reviewer.
mode: subagent
model: openrouter/z-ai/glm-5.1
temperature: 0.1
---
```

```yaml
---
description: Experimental second opinion.
mode: subagent
model: openrouter/minimax/minimax-m2.7
temperature: 0.15
---
```

## Local / Ollama-style example

```yaml
model: ollama/qwen3-coder
```

or:

```yaml
model: ollama/deepseek-v4-pro
```

Use whatever model IDs your local/Ollama Cloud provider exposes.
