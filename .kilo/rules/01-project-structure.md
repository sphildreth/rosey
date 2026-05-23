# Rosey Project Structure Rules

- `/docs` is for user-facing documentation only.
- `/design` is for PRDs, SPECs, ADRs, migration notes, prompts, and coding-agent materials.
- ADRs live in `/design/adr`.
- Coding-agent prompts live in `/design/prompts`.
- Do not place design rationale or agent prompts in `/docs`.
- Keep crate responsibilities separated:
  - `rosey-core`: pure domain behavior.
  - `rosey-fs`: filesystem and transfer behavior.
  - `rosey-metadata`: metadata providers/cache.
  - `rosey-cli`: command-line interface.
  - `rosey-tui`: terminal UI.
