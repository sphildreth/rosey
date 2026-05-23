# Optional Kilo Custom Modes Recommendation

This repo does not require custom modes to be productive. Start with:

```text
AGENTS.md
.kilo/rules/*.md
.kilo/skills/*/SKILL.md
design/prompts/agents/*.md
```

If custom modes are useful later, create narrow modes around these jobs:

1. `rosey-phase-executor`
   - Implements one migration slice at a time.
   - Must run cargo fmt/test/clippy.

2. `rosey-reviewer`
   - Reviews crate boundaries, safety, and tests.
   - Does not implement large features.

3. `rosey-test-runner`
   - Runs gates and fixes minimal failures.

4. `rosey-docs-adr`
   - Updates `/design`, ADRs, and prompts.
   - Does not change source unless asked.

Keep custom modes minimal. Overly broad mode prompts often make agents slower and more error-prone.
