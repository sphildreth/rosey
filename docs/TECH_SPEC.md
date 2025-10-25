
# Rosey — Technical Specification (PySide6)
Version: 1.0 • Date: 2025-10-25

## 1) Stack
- **Language:** Python 3.11+
- **UI:** PySide6 (Qt 6, Widgets)
- **Concurrency:** QThreadPool + QRunnable; signals/slots for progress
- **HTTP:** httpx.Client (synchronous, thread-safe)
- **Cache:** SQLite (sqlite3 / SQLAlchemy or sqlmodel)
- **Config:** pydantic model → `rosey.json` (in app data dir); `keyring` for secrets
- **Logging:** logging + RotatingFileHandler (in app data dir)
- **Packaging:** PyInstaller (`--windowed`) for Windows & Linux

## 2) Architecture
```
src/
  app.py            # Qt bootstrap, theme setup
  ui_main.py        # MainWindow: controls, table view, toolbar/status
  viewmodels.py     # QAbstractTableModel for candidates
  tasks/
    scan_task.py    # QRunnable: scan + offline identify
    move_task.py    # QRunnable: plan application (move/copy+delete)
    lookup_task.py  # QRunnable: online provider enrichment
  engine/
    scanner.py      # enumerate FS, detect network share vs local
    nfo.py          # parse .nfo
    patterns.py     # filename/folder regexes
    identify.py     # Movie/TV/Unknown classification
    providers/
      tmdb.py       # TMDB adapter, cache, rate limit
      tvdb.py       # optional TVDB adapter
    score.py        # confidence + reasons
    planner.py      # Jellyfin naming, sanitize
    mover.py        # atomic rename or copy+delete + progress + cancel
    config.py       # pydantic settings, load/save
    utils.py        # path, long‑path helpers, throttling
  assets/           # icons, qss
tests/              # pytest: unit + parity
```

## 3) UI behavior
- Build the grid using `QAbstractTableModel` with roles for text + checkbox state.
- Use `QSortFilterProxyModel` for filtering (All/Movies/TV/Unknown/Green) and sorting.
- Theme via QSS (two files: `light.qss`, `dark.qss`), load at runtime with a toggle.
- Status bar: counts, progress bar, error indicator, **View Log** action.
- Don’t block: tasks run in QThreadPool, send progress via `Signal`.

## 4) Data models
```python
from dataclasses import dataclass
from enum import Enum, auto
from typing import Optional, List

class MediaType(Enum):
    UNKNOWN = auto()
    MOVIE = auto()
    TV = auto()

@dataclass
class ProviderMatch:
    provider: str
    id: str
    canonical_title: str
    year: Optional[int] = None

@dataclass
class Candidate:
    source_path: str
    type: MediaType   # Enum for type safety
    proposed_name: str
    confidence: int
    reasons: List[str]
    destination: str
    season: Optional[int] = None
    episode: Optional[int] = None
    episode_title: Optional[str] = None
    online_match: Optional[ProviderMatch] = None
```

## 5) Engine essentials
### Scanner
- Use `os.scandir()` recursively; gather candidate roots (movie folders, show/season, lone video files).
- **Robust network share detection:**
  - Windows: Use `WNetGetConnection` via `ctypes` to check if a path is on a mapped network drive.
  - Linux: Check filesystem type via `findmnt -T <path>` or by parsing `/proc/mounts`.
- Concurrency knobs: `max_workers_local=8`, `max_workers_network=3` (configurable).

### Patterns & NFO
- Regex: `S(?P<s>\d{1,2})E(?P<e>\d{1,3})`, `(?P<s>\d{1,2})x(?P<e>\d{1,3})`, `\d{4}-\d{2}-\d{2}`.
- Folder: `Title (Year)`, `Show/Season NN`.
- Parse `.nfo` via `xml.etree.ElementTree`, tolerant to whitespace/encoding.

### Online lookups
- **httpx.Client** with limits; semaphore for rate limit; retries with exponential backoff on 429/5xx.
- Cache: SQLite tables keyed by (`kind`,`key`) → JSON blob + `updated_at`.
- Respect `language/region` for titles.
- Attribution: “Data provided by TMDB/TVDB” in About.

### Scoring
- Additive scoring with reasons; thresholds from PRD (Green ≥70, etc.).
- Online match boosts: +50 for exact ID; +30 for Title+Year; +20 for episode title match.

### Planner
- Build destination paths per Jellyfin rules; **preserve extension**.
- Sanitize invalid characters: `<>:"/\|?*` and trailing spaces/dots; collapse spaces.

### Mover
- **Transactional moves:** For multi-file operations (e.g., a TV season), treat the move as a single transaction. If any file fails to copy, abort the operation and delete any partially copied files from the destination to ensure a clean rollback. Only delete source files after all copies are verified.
- **Sidecar files:** When a media file is moved, the mover will also find and move any associated sidecar files (e.g., `.srt`, `.nfo`, `.jpg`) that share the same base filename.
- **Same volume:** `os.replace()` (atomic rename).
- **Cross volume:** stream copy (`shutil.copy2`) then `os.remove()`; report bytes for progress.
- Bounded concurrency; cancellation token checks between files.
- Conflict policy: Skip / Replace / Keep Both (`name (1).ext`).

## 6) Config & Logging
### Config (`rosey.json`)
```json
{
  "sourceRoot": "D:/Downloads/Unsorted",
  "moviesTarget": "E:/Media/Movies",
  "tvTarget": "E:/Media/TV",
  "theme": "system|light|dark",
  "useOnlineLookups": true,
  "providers": {
    "priority": ["TMDB","TVDB"],
    "tmdb": {"apiKey":"", "language":"en-US", "region":"US"},
    "tvdb": {"apiKey":"", "language":"en"}
  },
  "cache": {"dir":"./cache","ttlDays":30},
  "network": {"maxParallelLocal":8,"maxParallelNetwork":3,"retryCount":3},
  "dryRun": false
}
```
- Use a library like `appdirs` to store config and logs in a standard, platform-specific application data directory (e.g., `~/.config/rosey`).
- **Secrets:** Use the `keyring` library to store API keys in the OS credential manager (Keychain, Windows Credential Manager, Secret Service). Avoid storing secrets in the JSON file.

### Logging
- `logging` with RotatingFileHandler (e.g., 5×5MB), path in standard app data log directory.
- Redact API keys; log src→dest, durations, bytes moved, and errors.

## 7) Performance
- UI thread: **no blocking**; all heavy work in QThreadPool.
- Scans avoid unnecessary `stat()` calls; batch directory walks.
- Network shares default to lower concurrency; user‑tunable.
- Large file copies use large buffers; on Linux consider `sendfile` fast‑path (optional).

## 8) Testing
- **Unit:** regex parsing, NFO parsing, sanitization, planner.
- **Integration:** temp trees → plan; move dry‑run; conflict handling.
- **Parity (optional):** golden JSONs from C# engine and pytest comparisons.

## 9) Packaging
- PyInstaller single‑folder bundles with icons (ICO/PNG).
- Windows: create `.ico` multi‑size; Linux: install `.desktop` and PNGs (16–512).

## 10) Theming
- QSS for light/dark themes; load chosen stylesheet at startup.
- Save last theme in config; allow runtime switch without restart if possible.

## 11) Accessibility & i18n
- Keyboard navigation for all actions; tab order; tooltips.
- Strings collected for translation; configurable UI language in future.
