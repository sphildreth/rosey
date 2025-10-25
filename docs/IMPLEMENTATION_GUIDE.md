# Rosey — Implementation Guide (Coding Agents)

This guide translates the PRD into an agent-driven implementation plan. It defines phases with clear acceptance criteria, agent roles, guardrails, data contracts, testing, and quality gates. No timeframes are included.

> For coding agents (Copilot/Claude):
> See **docs/AI_AGENT_PLAYBOOK.md** for exact prompts, response format (unified diffs only),
> and step-by-step tasks (shell → tree filtering → scan threading → config dialog → detail dialog → mover).


## Phase Map

- [x] [Phase A — Planning & Guardrails](#phase-a)
- [x] [Phase B — M1: Scan + Identify (offline) + Score + Plan + CLI + Minimal UI](#phase-b)
- [x] [Phase C — M2: Move Engine + Conflicts + Sidecars + Logging + Progress/Cancel](#phase-c)
- [ ] [Phase D — M3: Online Lookups (TMDB/TVDB) + Cache + Settings UI](#phase-d)
- [ ] [Phase E — M4: Packaging (Win/Linux) + Icons + Docs + First Binaries](#phase-e)
- [ ] [Continuous — Tests & Docs + Observability + Perf/Stress + Cross-Platform CI](#continuous)

---

## Coding Agent Template
```
Task: Implement only the unchecked items in Phase D (see IMPLEMENTATION_GUIDE.md phase-b).
Scope: Modify only src/rosey/**, tests/**, docs/**, and packaging configs; keep the diff small and self-contained.
Requirements: Add/update tests so the phase’s acceptance criteria pass; UI work must stay responsive (threads); online provider calls are opt-in and use recorded fixtures by default; dry-run is the default for move operations.
Quality gates: pytest all green, ruff clean, mypy clean, UI smoke run OK.
Project: Rosey (PySide6, Qt Widgets, QSS). See docs/PRD.md, docs/TECH_SPEC.md, docs/mockups/.
Run: python -m rosey.app
Tests: pytest -q
Lint: ruff check . ; Format: black .
Output: Code + tests + a brief summary of changes and how you verified them (commands and results). If blocked, state the minimal decision needed to proceed.

After an item’s changes pass all quality gates, update docs/IMPLEMENTATION_GUIDE.md:
In Phase D section, change that item’s checkbox to [x].
If every item in Phase D is now checked, also check the corresponding Phase line in the Phase Map at the top.
Include IMPLEMENTATION_GUIDE.md in your unified diff when you update checkboxes.
Do not check items that are only partially complete; if partially done, leave as [ ] and mention “partial” in your summary.
Only mark items completed if you added/changed code and tests that verify the acceptance criteria

Follow AI_AGENT_PLAYBOOK.md. Return unified diffs only. Modify only the files I name.
If the code fails to run or tests fail, I will paste errors; respond with the smallest possible fix diff.

```

---

## Principles

- Privacy-first: offline by default; online lookups opt-in and budgeted.
- Safety-first I/O: transactional moves; dry-run available; explicit user approvals.
- Small verifiable steps: agents produce small diffs with tests and typed contracts.
- Deterministic orchestration: milestone tasks planned in advance, auto-gated by CI.
- Cross-platform: Windows/Linux path rules, concurrency tuning, and packaging.

---

## Orchestration Overview

- Orchestrator (deterministic supervisor) plans tasks from PRD/TECH_SPEC and routes them to specialized agents.
- Write-scope is restricted (e.g., `src/rosey/**`, `tests/**`, `docs/**`, packaging configs).
- Merge is blocked unless tests, lint, type checks, and UI smoke checks pass.
- Human approval gates at the end of each sub-batch (e.g., “Identifier + tests”).

---

## Agent Roles (inputs → outputs → definition of done)

- Spec-to-Issues Agent
  - Inputs: `PRD.md`, `TECH_SPEC.md`, mockups, repo state.
  - Outputs: structured issues/tasks per phase with acceptance criteria.
  - DoD: Issues reference PRD FRs and include testable AC and file write-scope.

- Library/CLI Agent (foundation for M1)
  - Inputs: task spec for scan/identify/score/plan.
  - Outputs: `src/rosey/**` modules and `tests/**`, a minimal CLI with dry-run.
  - DoD: Tests pass; CLI prints structured results; types and ruff clean.

- Scanner Agent
  - Inputs: Source path(s), concurrency policy (local vs network).
  - Outputs: enumerated file records with errors logged but non-fatal.
  - DoD: Recurses large trees without UI blockage; handles permission errors.

- Identifier Agent (offline-first)
  - Inputs: file/folder names, `.nfo` files.
  - Outputs: parsed media items (movie/show/episode/unknown) with reasons.
  - DoD: Patterns supported (SxxEyy, 1x02, S01E01-E02, Part N, YYYY, YYYY-MM-DD); malformed → Unknown with reason.

- Scorer Agent
  - Inputs: identification reasons/metadata.
  - Outputs: confidence 0–100 with reasons; thresholds Green≥70, Yellow 40–69, Red<40.
  - DoD: “Select All Green” logic covered by tests.

- Path Planner Agent
  - Inputs: media items (+ online IDs when present).
  - Outputs: destination paths per Jellyfin rules; sanitized for Windows/Linux.
  - DoD: Multi-episode/multipart/specials handled; reserved names sanitized.

- Mover Agent (transactional)
  - Inputs: planned moves, filesystem topology (same vs cross-volume).
  - Outputs: committed moves with rollback on failure; sidecars co-moved.
  - DoD: Atomic rename on same volume; cross-volume copy→verify→quarantine→commit; property tests inject failures.

- Online Providers Agent
  - Inputs: API keys, cache, language/region.
  - Outputs: TMDB/TVDB metadata with caching and rate limiting.
  - DoD: Recorded-fixture tests; offline graceful degradation; errors logged. Exposes a "Discover" trigger for UI actions and responds with cached/budgeted lookups.

- PySide6 UI Agent
  - Inputs: core library API.
  - Outputs: single-window app with tree (left) + grid (right), theme toggle, filters, selection helpers; conflict dialog; progress; context menu on Library Tree with a Discover action (M3).
  - DoD: Long-running work in threads; UI responsive during scan/move.

- Config & Logging Agent
  - Inputs: config schema, log policy.
  - Outputs: `rosey.json` (paths, keys, theme, concurrency, cache TTL); rotating file logs (redacted); on-screen status pane.
  - DoD: Load/save config; secrets never printed; logs searchable and rotated.

- Packaging Agent
  - Inputs: build spec for Win/Linux.
  - Outputs: PyInstaller specs, icons, smoke tests.
  - DoD: Binaries build on CI runners; launch and exit cleanly; dry-run works.

- Test Writer/Evaluator Agent
  - Inputs: FRs/ACs and code under test.
  - Outputs: unit, property, and stress tests; synthetic file trees; recorded fixtures.
  - DoD: Meaningful coverage; regression suite; green-before-merge.

- Docs Agent
  - Inputs: current feature set and flags.
  - Outputs: updated README and docs, troubleshooting, and usage examples.
  - DoD: Docs reflect current behavior and flags (e.g., dry-run default).

---

## Data Contracts (pydantic-style)

These models keep modules and agents aligned. Names are examples; adapt as needed.

```python
from typing import List, Optional, Dict
from pydantic import BaseModel

class MediaItem(BaseModel):
    kind: str  # "movie" | "show" | "episode" | "unknown"
    source_path: str
    title: Optional[str] = None
    year: Optional[int] = None
    season: Optional[int] = None
    episodes: Optional[List[int]] = None  # e.g., [1] or [1,2]
    part: Optional[int] = None            # e.g., 1, 2 for multipart
    date: Optional[str] = None            # YYYY-MM-DD
    sidecars: List[str] = []
    nfo: Dict[str, Optional[str]] = {}    # ids/title/year/episode_title

class IdentificationResult(BaseModel):
    item: MediaItem
    reasons: List[str]
    online_metadata: Optional[Dict] = None
    errors: List[str] = []

class Score(BaseModel):
    confidence: int  # 0–100
    reasons: List[str]

class MovePlan(BaseModel):
    destination_paths: List[str]
    conflicts: List[Dict] = []
    preflight: Dict[str, bool] = {"free_space_ok": True, "perms_ok": True, "path_len_ok": True}
    dry_run: bool = True

class MoveResult(BaseModel):
    success: bool
    details: Dict[str, List[str]]  # moved/skipped/replaced/kept_both
    rollback_performed: bool = False
    errors: List[str] = []
```

---

## Tooling and Guardrails

- Language/Runtime: Python 3.11; PySide6 for UI.
- Core libs: httpx (providers), pydantic (models), regex helpers, optional diskcache/SQLite for caching.
- Quality: pytest + hypothesis, ruff, black, mypy, pre-commit hooks.
- Secrets: `.env`/OS keyring; never committed; redacted in logs.
- Network: live provider calls behind explicit flag; recorded fixtures used in CI by default.
- Write-scope: agents can only modify approved directories.
- Diff size: prefer small changes (<~500 LOC or <~5 files) unless approved by supervisor.
- Rate limits: budget per test run; exponential backoff; circuit breaker on repeated failures.

---

## CI Quality Gates (PASS criteria)

- Build: package/binaries build successfully; CLI entry point runs; UI smoke script launches and quits.
- Lint/Typecheck: ruff and mypy pass with no new warnings.
- Tests: unit + property tests pass; synthetic stress tests for large scans/moves.
- Binary Smoke (CI runners): dry-run scan/plan works on Windows/Linux.

---

## Tests to Write First (exemplars)

- Scanner
  - [ ] Recurses ~50k synthetic files without blocking UI thread (tested with background worker).
  - [ ] Logs permission errors without crashing.
- Identifier
  - [ ] Patterns: SxxEyy, 1x02, S01E01-E02, Part N, YYYY, YYYY-MM-DD.
  - [ ] NFO with IDs overrides filename ambiguity; malformed NFO → Unknown with reason.
- Scorer
  - [ ] Deterministic thresholds: Green≥70, Yellow 40–69, Red<40; “Select All Green” only ≥70.
- Path Planner
  - [ ] Windows/Linux-safe; reserved names sanitized; Specials → Season 00; multipart and multi-episode naming rules.
- Mover
  - [ ] Transactional: injected failures (size mismatch, mid-copy failure, permission error) trigger rollback and leave destination clean.
  - [ ] Sidecar co-move; conflict suffix “(1)” for Keep Both.
- Online Providers
  - [ ] Cache hit/miss; budgeted calls; graceful offline behavior with clear logs.
  - [ ] UI Discover action triggers provider lookups when enabled; disabled/offline state handled gracefully.
- UI
  - [ ] Filters and selection helpers work; progress updates; responsiveness during long ops.

---

## Phase Checklists

<a id="phase-a"></a>
### Phase A — Planning & Guardrails

- [x] Create structured issues from PRD/TECH_SPEC with AC and write-scope.
- [x] Establish repo structure: `src/rosey/**`, `tests/**`, `docs/**`.
- [x] Add pre-commit: ruff, black, mypy; pytest config with hypothesis.
- [x] Configure CI: lint, typecheck, tests, UI smoke, packaging lanes.
- [x] Define config schema (`rosey.json`) and logging/redaction policy.
- [x] Decide caching backend and recording strategy for provider fixtures.
- [x] Document development policies (write-scope, diff size, approval gates).

<a id="phase-b"></a>
### Phase B — M1: Core Library + CLI + Minimal UI

- [x] Implement scanner with concurrency knobs and error logging.
- [x] Implement offline identifier (nfo + filename/folder patterns).
- [x] Implement scorer (0–100) with explicit reasons and thresholds.
- [x] Implement path planner per Jellyfin rules and sanitization.
- [x] Provide CLI to run scan→identify→score→plan (dry-run by default).
- [x] Minimal PySide6 UI: tree (left), grid (right), filters, Select All Green.
- [x] Background workers for scanning; UI remains responsive.
- [x] Tests: scanner/identifier/scorer/planner + UI headless checks.

<a id="phase-c"></a>
### Phase C — M2: Move Engine + Conflicts + Logging + Progress/Cancel

- [x] Transactional move engine (rename same volume; copy→verify→quarantine→commit across volumes).
- [x] Sidecar discovery and co-move.
- [x] Conflict dialog (Skip/Replace/Keep Both); suffix “(1)” policy.
- [x] Preflight checks: free space, path length, permissions.
- [x] Progress UI, cancel, and rollback on failure.
- [x] Rotating logs (file) + status pane (redacted).
- [x] Property-based tests inject failures; rollback guarantees verified.

<a id="phase-d"></a>
### Phase D — M3: Online Lookups + Cache + Settings UI

- [ ] TMDB primary + TVDB optional; localization (language/region).
- [ ] Disk-backed cache and rate limiting with backoff.
- [ ] Settings UI for API keys, cache TTL, concurrency, language/region, dry-run.
- [ ] Recorded-fixture tests; live calls opt-in with budget; graceful degradation.
- [ ] Library Tree context menu: Discover action — runs in background thread; respects rate limit/cache; shows non-blocking status; disabled when providers are off; errors logged.

<a id="phase-e"></a>
### Phase E — M4: Packaging + Icons + Docs + Binaries

- [ ] PyInstaller specs for Windows/Linux; app icons.
- [ ] Post-build smoke tests: launch, pick folders, scan, dry-run move, exit.
- [ ] User-facing docs: setup, troubleshooting (paths, permissions, network shares).

<a id="continuous"></a>
### Continuous — Tests, Docs, Observability, Performance

- [ ] Coverage thresholds enforced; regression suite maintained.
- [ ] Synthetic stress for large libraries; concurrency tuning for network shares.
- [ ] Logging/telemetry policy respected (no telemetry; provider calls explicit).
- [ ] Cross-platform CI lanes for Windows and Linux packaging.

---

## Repository Conventions

- Code: `src/rosey/**` with clear module boundaries: scanner, identifier, scorer, planner, mover, providers, ui, config, logging, utils.
- Tests: `tests/**` mirroring module layout; golden fixtures for identifier; property tests for mover.
- Docs: `docs/**` including PRD, TECH_SPEC, and this guide.
- Config: `rosey.json` in user-space; secrets via `.env`/keyring.

---

## Risks & Mitigations (agent-specific)

- Overreach edits → restricted write-scope; small diffs; pre-commit hooks; supervisor approval.
- Flaky UI tests → focus on model/logic tests; keep UI tests thin (signals/slots, model changes).
- Provider variability and rate limits → recorded fixtures by default; live tests gated and budgeted; exponential backoff.
- Cross-volume and network errors → transactional engine with rollback; retries with backoff; clear user feedback.
- Windows path quirks → centralized path utils; Windows CI lane; long-path normalization.

---

## Definition of Done (per phase)

- Phase A: Issues + policies + CI in place; write-scope enforced; docs updated.
- Phase B: Core library + CLI + minimal UI; tests green; UI responsive; dry-run default.
- Phase C: Transactional mover with conflicts/sidecars; progress/cancel; rollback tests pass.
- Phase D: Providers integrated with cache and settings; recorded fixtures; graceful offline behavior.
- Phase E: Packaged binaries for Win/Linux; smoke tests pass; docs ready.
