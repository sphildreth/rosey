
# AI Agent Playbook (PySide6) — Rosey
**Purpose:** Make Copilot/Claude reliably produce *running* code by constraining scope, format, and acceptance criteria. This complements `docs/IMPLEMENTATION_GUIDE.md`.

## House Rules (always include in your prompt)
- **Scope:** Only modify the files I name. No broad refactors. No touching packaging unless asked.
- **Output format:** Return a **single unified diff** per file you change. No prose above/below the diff.
- **Seed UI:** The app must launch and show **seed rows** so we can visually confirm.
- **Threading:** Long work via **QThreadPool + QRunnable** with `Signal`s. Never block the UI thread.
- **Tests:** If I paste errors, respond with the **minimal fix diff**.
- **IDs:** Use control IDs from `UI_MOCKUPS.md` / `UI_COMPONENT_IDS.md` when relevant.

## Project facts (for the agent)
- **Stack:** Python 3.11+, PySide6 (Qt Widgets), QSS theming, pytest, ruff, black.
- **UI gist:** Action bar (Run/Config/Scan/Move/Select Green/Clear), left **tree** (`treeLibrary`), right **grid** (`tableDetails`), bottom **activity log** (`textActivity`), **status bar**.
- **Dialogs:** `dlgConfig` (settings), `dlgDetail` (media details, preview stub).
- **Docs:** `docs/PRD.md`, `docs/TECH_SPEC.md`, `docs/mockups/*`.
- **Run:** `python -m rosey.app` (via VS Code launch.json).

## Command snippets (for the agent to reference)
- **Run app:** `python -m rosey.app`
- **Run tests:** `pytest -q`
- **Lint/format:** `ruff check .` / `black .`

## Step Plan (ask one step at a time)
### Step 1 — runnable shell + seed rows
**Files to touch:** `rosey/app.py`, `rosey/ui_main.py`, `rosey/viewmodels.py`
**Task:** Build a main window with action bar, splitters (tree left / grid right, log bottom), status bar. Implement `QAbstractTableModel` with 3 seed rows. Wire **Scan** to append an activity log line.
**Acceptance:** F5 launches; 3 rows visible; clicking rows doesn’t error.

**Prompt template:**
> Create/modify only `rosey/app.py`, `rosey/ui_main.py`, `rosey/viewmodels.py`.
> Requirements: PySide6, QSplitter layout (treeLibrary, tableDetails, textActivity), QStatusBar, seed rows in table model, “Scan” appends an activity message.
> Output: unified diffs only.

### Step 2 — tree selection filters grid
**Files:** `rosey/viewmodels.py` (add models), possibly `rosey/ui_main.py`.
**Task:** Implement a simple tree model (QStandardItemModel is fine) with top nodes **Shows** and **Movies** and a few children. Use `QSortFilterProxyModel` to filter grid rows based on tree selection.
**Acceptance:** Selecting a node filters rows; clearing selection shows all.

### Step 3 — threading scaffold for Scan
**Files:** `rosey/tasks/scan_task.py`, `rosey/ui_main.py`.
**Task:** Implement `ScanTask(QRunnable)` that emits `progress(str)` and `completed(list)` signals. Hook **Scan** to run this task and populate the model with dummy items on `completed`.
**Acceptance:** UI responsive; activity log shows progress; grid updates on completion.

### Step 4 — configuration dialog
**Files:** `rosey/ui_config.py`, `rosey/app.py` (menu/button handler).
**Task:** Dialog to edit Source/Movies/TV paths, theme (system/light/dark), online lookups toggle, concurrency knobs, API keys. Persist to `rosey.json` in project dir. Allow theme switch at runtime.
**Acceptance:** Save/Load works; theme applies; settings persist across runs.

### Step 5 — detail dialog & preview stub
**Files:** `rosey/ui_detail.py`, `rosey/viewmodels.py` (row → dto).
**Task:** Double‑click row to open details. Show title/type/confidence/source/destination/reasons and a stacked preview area (video/image/text). Use placeholder label for now.
**Acceptance:** Double‑click opens dialog without errors.

### Step 6 — move task scaffold
**Files:** `rosey/engine/mover.py`, `rosey/tasks/move_task.py`, `rosey/ui_main.py`.
**Task:** Implement safe move plan execution: `os.replace` when same volume, else `shutil.copy2` + `os.remove`. Bounded concurrency; emit progress; conflict policy placeholders (Skip/Replace/Keep Both). Wire **Run** and **Move Selected**.
**Acceptance:** Dry‑run mode works; progress updates; activity log lines.

## Response Format Requirement (copy verbatim into prompts)
> **Return only unified diffs.** For each changed file, start with a line like `*** Begin Patch` and end with `*** End Patch`. Use standard unified diff headers (`--- a/file`, `+++ b/file`). No extra commentary.

## Minimal seed row schema (for consistency)
```python
{
  "key": "tv:the_office_s02e01",
  "checked": false,
  "type": "TV",
  "name": "The Office — S02E01 — The Dundies",
  "season": 2, "episode": 1,
  "confidence": 90,
  "dest": "TV/The Office/Season 02/The Office - S02E01 - The Dundies.mkv",
  "source": "/path/to/file.mkv",
  "reasons": ["TMDB match","SxxEyy parse"]
}
```

## Troubleshooting policy
- If I paste a traceback/build error, **do not** rewrite files wholesale. Produce the **smallest fix**.
- If a control name mismatches, align to IDs in `UI_COMPONENT_IDS.md`.
- If UI fails to render rows, verify: public properties, model row/column counts, and view `setModel` calls.

## Done Definition (per step)
- App runs without exceptions.
- Feature works with seed data.
- Lint: `ruff check .` passes (or minimal ignores).
- Optional: add a tiny `pytest-qt` test when feasible.
