# Rosey — Structured Issues & Tasks

This document tracks implementation tasks derived from PRD and TECH_SPEC.

## Phase A — Planning & Guardrails ✅ COMPLETE

### Acceptance Criteria
- [x] Repository structure established
- [x] Pre-commit hooks configured
- [x] CI pipeline configured
- [x] Config schema defined
- [x] Caching backend decided
- [x] Recording strategy documented
- [x] Development policies documented

**Status**: Complete (2025-10-25)
**Summary**: See `docs/PHASE_A_SUMMARY.md`

---

## Phase B — M1: Core Library + CLI + Minimal UI

### Module: Scanner

**Task B.1**: Implement file scanner with concurrency
- **AC**:
  - [ ] Recursively scans source directory
  - [ ] Configurable concurrency (local vs. network)
  - [ ] Handles permission errors gracefully (log, don't crash)
  - [ ] Returns list of file paths with metadata
  - [ ] Tests: 50k synthetic files, permission errors
- **Write-scope**: `src/rosey/scanner/`, `tests/unit/test_scanner.py`
- **Dependencies**: None
- **Tests**: Property test for large trees, unit test for errors

**Task B.2**: Add background scanning with QThreadPool
- **AC**:
  - [ ] Scanner runs in background thread
  - [ ] Emits progress signals (files scanned, current file)
  - [ ] UI remains responsive during scan
  - [ ] Cancel operation supported
- **Write-scope**: `src/rosey/tasks/scan_task.py`, `tests/ui/test_scan_task.py`
- **Dependencies**: B.1
- **Tests**: pytest-qt test for signals, responsiveness

---

### Module: Identifier (Offline)

**Task B.3**: Implement filename/folder pattern matching
- **AC**:
  - [ ] Patterns: SxxEyy, 1x02, S01E01-E02, Part N, YYYY, YYYY-MM-DD
  - [ ] Multi-episode detection (S01E01-E02 → episodes=[1,2])
  - [ ] Multipart detection (Part 1 → part=1)
  - [ ] Title extraction from folders and filenames
  - [ ] Tests: All listed patterns, edge cases, malformed inputs
- **Write-scope**: `src/rosey/identifier/patterns.py`, `tests/unit/test_identifier_patterns.py`
- **Dependencies**: None
- **Tests**: Hypothesis property tests for pattern variations

**Task B.4**: Implement NFO parsing
- **AC**:
  - [ ] Parse XML NFO files
  - [ ] Extract: TMDB/TVDB IDs, title, year, episode title
  - [ ] Handle malformed NFO gracefully (log, fallback to Unknown)
  - [ ] Prefer NFO IDs over filename when available
  - [ ] Tests: Valid NFO, malformed NFO, missing fields
- **Write-scope**: `src/rosey/identifier/nfo.py`, `tests/unit/test_identifier_nfo.py`
- **Dependencies**: None
- **Tests**: Unit tests with fixture NFO files

**Task B.5**: Integrate identifier components
- **AC**:
  - [ ] Combine NFO + filename + folder patterns
  - [ ] Return `IdentificationResult` with reasons
  - [ ] Classify as movie/show/episode/unknown
  - [ ] Tests: Integration test with synthetic file tree
- **Write-scope**: `src/rosey/identifier/__init__.py`, `tests/integration/test_identifier.py`
- **Dependencies**: B.3, B.4
- **Tests**: Integration test with mixed scenarios

---

### Module: Scorer

**Task B.6**: Implement confidence scoring
- **AC**:
  - [ ] Score 0–100 based on identification signals
  - [ ] Thresholds: Green ≥70, Yellow 40–69, Red <40
  - [ ] Reasons: "NFO with ID", "Pattern match", "Ambiguous"
  - [ ] Deterministic: same input → same score
  - [ ] Tests: Each confidence band, edge cases at thresholds
- **Write-scope**: `src/rosey/scorer/__init__.py`, `tests/unit/test_scorer.py`
- **Dependencies**: B.5
- **Tests**: Unit tests for score calculation, threshold boundaries

---

### Module: Path Planner

**Task B.7**: Implement Jellyfin path rules
- **AC**:
  - [ ] Movie: `Movies/Title (Year) [ExternalId]/Title (Year).ext`
  - [ ] TV: `Shows/Show Title [Id]/Season 01/Show Title - S01E02 - Episode Title.ext`
  - [ ] Multi-episode: `Show - S01E01-E02 - Title.ext`
  - [ ] Multipart: `Show - S02E03 Part 1.ext`
  - [ ] Specials: Season 00
  - [ ] Tests: All naming patterns, reserved names, long paths
- **Write-scope**: `src/rosey/planner/__init__.py`, `tests/unit/test_planner.py`
- **Dependencies**: B.5
- **Tests**: Unit tests for each pattern, property tests for sanitization

**Task B.8**: Implement path sanitization (Windows/Linux)
- **AC**:
  - [ ] Replace invalid characters (Windows: `<>:"/\|?*`, Linux: `/`)
  - [ ] Handle reserved names (Windows: CON, PRN, AUX, etc.)
  - [ ] Trim trailing spaces and dots
  - [ ] Validate path length (Windows: 260, extended: 32767)
  - [ ] Tests: Reserved names, invalid chars, long paths
- **Write-scope**: `src/rosey/utils/paths.py`, `tests/unit/test_utils_paths.py`
- **Dependencies**: None
- **Tests**: Property tests for sanitization, platform-specific tests

---

### Module: CLI

**Task B.9**: Implement CLI for scan→identify→score→plan
- **AC**:
  - [ ] Scan source directory
  - [ ] Identify media items
  - [ ] Score confidence
  - [ ] Plan destination paths
  - [ ] Output structured JSON or human-readable format
  - [ ] Dry-run by default, `--execute` flag for actual moves
  - [ ] Tests: CLI invocation, output format, flags
- **Write-scope**: `src/rosey/cli.py`, `tests/integration/test_cli.py`
- **Dependencies**: B.1, B.5, B.6, B.7
- **Tests**: Integration test with synthetic file tree, CLI args

---

### Module: Minimal UI

**Task B.10**: Create main window layout
- **AC**:
  - [ ] Action bar: Scan, Config, Move, Select Green, Clear
  - [ ] Splitters: Tree (left) | Grid (right), Log (bottom)
  - [ ] Status bar
  - [ ] Theme toggle (system/light/dark)
  - [ ] Tests: UI smoke test (launch, show, close)
- **Write-scope**: `src/rosey/ui/main_window.py`, `tests/ui/test_main_window.py`
- **Dependencies**: None
- **Tests**: pytest-qt test for layout

**Task B.11**: Implement table model for media items
- **AC**:
  - [ ] QAbstractTableModel with columns: checkbox, type, name, season, episode, confidence, destination, source
  - [ ] Seed rows for testing
  - [ ] Sorting and filtering
  - [ ] Tests: Model row/column count, data access
- **Write-scope**: `src/rosey/ui/models.py`, `tests/ui/test_models.py`
- **Dependencies**: None
- **Tests**: pytest-qt test for model

**Task B.12**: Implement tree model for library
- **AC**:
  - [ ] Tree structure: Shows / Movies with children
  - [ ] Selecting tree node filters grid
  - [ ] Tests: Tree selection, filtering
- **Write-scope**: `src/rosey/ui/models.py`, `tests/ui/test_models.py`
- **Dependencies**: B.11
- **Tests**: pytest-qt test for tree model

**Task B.13**: Wire Scan button to scanner
- **AC**:
  - [ ] Clicking Scan runs scanner in background
  - [ ] Progress signals update activity log
  - [ ] Completed signal populates grid
  - [ ] UI responsive during scan
  - [ ] Tests: Click button, verify model updated
- **Write-scope**: `src/rosey/ui/main_window.py`
- **Dependencies**: B.2, B.11
- **Tests**: pytest-qt test for scan flow

**Task B.14**: Implement filters (All/Green/Yellow/Red)
- **AC**:
  - [ ] Buttons filter grid by confidence
  - [ ] "Select All Green" checks items with confidence ≥70
  - [ ] Tests: Filter buttons, selection helper
- **Write-scope**: `src/rosey/ui/main_window.py`
- **Dependencies**: B.11
- **Tests**: pytest-qt test for filters

---

### Phase B Acceptance Criteria
- [ ] Scanner implemented with tests
- [ ] Identifier (offline) implemented with tests
- [ ] Scorer implemented with tests
- [ ] Path planner implemented with tests
- [ ] CLI functional for scan→identify→score→plan
- [ ] Minimal UI with tree, grid, filters
- [ ] Background workers for scanning
- [ ] All tests pass (estimated: 50+ tests)

**Status**: Not started
**Blocked by**: None (Phase A complete)

---

## Phase C — M2: Move Engine + Conflicts + Logging + Progress/Cancel

### Module: Mover

**Task C.1**: Implement transactional move engine
- **AC**:
  - [ ] Same volume: Atomic rename with `os.replace`
  - [ ] Cross volume: Copy → verify size → quarantine original → commit
  - [ ] Rollback on failure
  - [ ] Property tests: Inject failures (size mismatch, permission error)
  - [ ] Tests: Same volume, cross volume, failure scenarios
- **Write-scope**: `src/rosey/mover/__init__.py`, `tests/unit/test_mover.py`
- **Dependencies**: B.7
- **Tests**: Property-based tests with hypothesis

**Task C.2**: Implement sidecar co-move
- **AC**:
  - [ ] Discover sidecars: `.srt`, `.ass`, `.vtt`, `.nfo`, `.jpg`, `.png`
  - [ ] Move sidecars with media file
  - [ ] Rename sidecars to match destination
  - [ ] Tests: Sidecar discovery, co-move
- **Write-scope**: `src/rosey/mover/sidecars.py`, `tests/unit/test_mover_sidecars.py`
- **Dependencies**: C.1
- **Tests**: Unit tests with synthetic file tree

**Task C.3**: Implement conflict detection and resolution
- **AC**:
  - [ ] Detect destination file exists
  - [ ] Conflict policies: Skip, Replace, Keep Both
  - [ ] Keep Both: Append `(1)`, `(2)`, etc.
  - [ ] Tests: Each policy, multiple conflicts
- **Write-scope**: `src/rosey/mover/conflicts.py`, `tests/unit/test_mover_conflicts.py`
- **Dependencies**: C.1
- **Tests**: Unit tests for each policy

**Task C.4**: Implement conflict dialog UI
- **AC**:
  - [ ] Dialog shows conflicting files
  - [ ] User selects: Skip / Replace / Keep Both / Apply to All
  - [ ] Tests: Dialog interaction
- **Write-scope**: `src/rosey/ui/conflict_dialog.py`, `tests/ui/test_conflict_dialog.py`
- **Dependencies**: C.3
- **Tests**: pytest-qt test for dialog

**Task C.5**: Implement preflight checks
- **AC**:
  - [ ] Check free space on destination
  - [ ] Check path length (Windows: 260, Linux: 4096)
  - [ ] Check write permissions
  - [ ] Surface actionable errors
  - [ ] Tests: Each check, failure scenarios
- **Write-scope**: `src/rosey/mover/preflight.py`, `tests/unit/test_mover_preflight.py`
- **Dependencies**: C.1
- **Tests**: Unit tests with mocked filesystem

**Task C.6**: Implement progress and cancel UI
- **AC**:
  - [ ] Progress bar with file count and percentage
  - [ ] Cancel button stops current move and rolls back
  - [ ] Activity log shows each file moved/skipped
  - [ ] Tests: Progress updates, cancel mid-move
- **Write-scope**: `src/rosey/ui/progress_dialog.py`, `tests/ui/test_progress_dialog.py`
- **Dependencies**: C.1, C.4
- **Tests**: pytest-qt test for progress

---

### Module: Logging

**Task C.7**: Implement file logging with rotation
- **AC**:
  - [ ] Log to file with rotation (max size, backup count)
  - [ ] Redact API keys and secrets
  - [ ] Log levels: DEBUG, INFO, WARNING, ERROR, CRITICAL
  - [ ] Tests: Logging, rotation, redaction
- **Write-scope**: `src/rosey/utils/logging.py`, `tests/unit/test_utils_logging.py`
- **Dependencies**: None
- **Tests**: Unit tests for logging, redaction

**Task C.8**: Add activity log pane to UI
- **AC**:
  - [ ] Text widget shows session log
  - [ ] Auto-scroll to bottom
  - [ ] Clear button
  - [ ] Tests: Log messages appear
- **Write-scope**: `src/rosey/ui/main_window.py`
- **Dependencies**: C.7, B.10
- **Tests**: pytest-qt test for log pane

---

### Phase C Acceptance Criteria
- [ ] Transactional move engine with rollback
- [ ] Sidecar co-move
- [ ] Conflict detection and resolution
- [ ] Conflict dialog UI
- [ ] Preflight checks
- [ ] Progress and cancel UI
- [ ] File logging with rotation and redaction
- [ ] Activity log pane in UI
- [ ] All tests pass (estimated: 80+ tests)

**Status**: Not started
**Blocked by**: Phase B

---

## Phase D — M3: Online Lookups + Cache + Settings UI

### Module: Providers

**Task D.1**: Implement TMDB provider
- **AC**:
  - [ ] Search movies and TV shows
  - [ ] Fetch metadata by ID
  - [ ] Rate limiting (4 req/sec)
  - [ ] Language/region support
  - [ ] Tests: Recorded fixtures
- **Write-scope**: `src/rosey/providers/tmdb.py`, `tests/integration/test_providers_tmdb.py`
- **Dependencies**: None
- **Tests**: Integration tests with fixtures

**Task D.2**: Implement TVDB provider
- **AC**:
  - [ ] Search TV series
  - [ ] Fetch episodes by season
  - [ ] API key authentication
  - [ ] Tests: Recorded fixtures
- **Write-scope**: `src/rosey/providers/tvdb.py`, `tests/integration/test_providers_tvdb.py`
- **Dependencies**: None
- **Tests**: Integration tests with fixtures

**Task D.3**: Implement metadata cache
- **AC**:
  - [ ] DiskCache backend
  - [ ] TTL and size limits
  - [ ] Cache key generation
  - [ ] Tests: Cache hit/miss, eviction
- **Write-scope**: `src/rosey/providers/cache.py`, `tests/unit/test_providers_cache.py`
- **Dependencies**: D.1, D.2
- **Tests**: Unit tests for cache operations

**Task D.4**: Integrate providers into identifier
- **AC**:
  - [ ] Online lookups opt-in via config
  - [ ] Fallback to offline on failure
  - [ ] Cache responses
  - [ ] Tests: Online/offline modes, fallback
- **Write-scope**: `src/rosey/identifier/__init__.py`
- **Dependencies**: D.3, B.5
- **Tests**: Integration tests with mocked providers

---

### Module: Settings UI

**Task D.5**: Implement settings dialog
- **AC**:
  - [ ] Tabs: Paths, Behavior, Providers, Logging
  - [ ] Save/Cancel buttons
  - [ ] Validate inputs
  - [ ] Tests: Dialog interaction, validation
- **Write-scope**: `src/rosey/ui/settings_dialog.py`, `tests/ui/test_settings_dialog.py`
- **Dependencies**: None
- **Tests**: pytest-qt test for dialog

**Task D.6**: Add API key management
- **AC**:
  - [ ] Input fields for TMDB/TVDB keys
  - [ ] Store in OS keyring if available, else config file
  - [ ] Test button to verify API keys
  - [ ] Tests: Key storage, retrieval
- **Write-scope**: `src/rosey/utils/keyring.py`, `src/rosey/ui/settings_dialog.py`
- **Dependencies**: D.5
- **Tests**: Unit tests for keyring, UI test for dialog

---

### Phase D Acceptance Criteria
- [ ] TMDB provider with rate limiting
- [ ] TVDB provider
- [ ] Metadata cache with TTL
- [ ] Online lookups integrated into identifier
- [ ] Settings dialog with all config options
- [ ] API key management (keyring + fallback)
- [ ] All tests pass (estimated: 100+ tests)

**Status**: Not started
**Blocked by**: Phase B, Phase C

---

## Phase E — M4: Packaging + Icons + Docs + Binaries

### Module: Packaging

**Task E.1**: Create PyInstaller specs
- **AC**:
  - [ ] Windows spec: Single-file or one-dir bundle
  - [ ] Linux spec: AppImage or deb
  - [ ] Include icons and resources
  - [ ] Tests: Post-build smoke test
- **Write-scope**: `rosey.spec`, `rosey-linux.spec`, `scripts/build.sh`
- **Dependencies**: All previous phases
- **Tests**: Smoke test in CI

**Task E.2**: Add application icons
- **AC**:
  - [ ] Windows: `.ico` file
  - [ ] Linux: `.png` files (various sizes)
  - [ ] macOS: `.icns` file (future)
- **Write-scope**: `graphics/icons/`
- **Dependencies**: None
- **Tests**: Manual verification

**Task E.3**: CI packaging lanes
- **AC**:
  - [ ] Windows: Build `.exe` on Windows runner
  - [ ] Linux: Build AppImage on Ubuntu runner
  - [ ] Upload artifacts
  - [ ] Tests: Smoke test on built binaries
- **Write-scope**: `.github/workflows/package.yml`
- **Dependencies**: E.1, E.2
- **Tests**: CI job runs successfully

---

### Module: Documentation

**Task E.4**: Write user-facing documentation
- **AC**:
  - [ ] README: Installation, quick start
  - [ ] Setup guide: Paths, API keys
  - [ ] Troubleshooting: Common errors
  - [ ] FAQ
- **Write-scope**: `README.md`, `docs/USER_GUIDE.md`, `docs/TROUBLESHOOTING.md`
- **Dependencies**: All previous phases
- **Tests**: Manual review

**Task E.5**: Create release notes
- **AC**:
  - [ ] Changelog updated
  - [ ] GitHub release notes
  - [ ] Known issues documented
- **Write-scope**: `CHANGELOG.md`, GitHub release
- **Dependencies**: All previous phases
- **Tests**: Manual review

---

### Phase E Acceptance Criteria
- [ ] PyInstaller specs for Windows and Linux
- [ ] Application icons
- [ ] CI packaging lanes
- [ ] Packaged binaries (Windows `.exe`, Linux AppImage)
- [ ] Post-build smoke tests pass
- [ ] User-facing documentation complete
- [ ] Release notes published

**Status**: Not started
**Blocked by**: Phase B, Phase C, Phase D

---

## Continuous — Tests, Docs, Observability, Performance

### Ongoing Tasks

**Task X.1**: Maintain regression suite
- [ ] Add tests for bug fixes
- [ ] Update tests for feature changes
- [ ] Ensure coverage thresholds maintained

**Task X.2**: Performance tuning
- [ ] Profile large library scans (>10k files)
- [ ] Optimize hot paths
- [ ] Tune concurrency for network shares

**Task X.3**: Cross-platform testing
- [ ] Test on Windows 10, Windows 11
- [ ] Test on Ubuntu, Fedora, Arch
- [ ] Test with network shares (SMB, NFS, UNC)

**Task X.4**: Documentation maintenance
- [ ] Keep docs in sync with code
- [ ] Update troubleshooting for new errors
- [ ] Add examples for new features

---

## Issue Tracking Template

For each task, create a GitHub issue with:

**Title**: `[Phase X] Task X.Y: <Short Description>`

**Labels**: `phase-a`, `scanner`, `tests`, etc.

**Body**:
```markdown
## Summary
Brief description from task list above.

## Acceptance Criteria
- [ ] AC 1
- [ ] AC 2
- ...

## Write-Scope
- File 1
- File 2
- ...

## Dependencies
- #123 (Task X.Y)
- #456 (Task X.Z)

## Tests Required
- Test type 1
- Test type 2

## Definition of Done
- [ ] Code written and tested locally
- [ ] Tests pass (pytest)
- [ ] Lint passes (ruff)
- [ ] Type check passes (mypy)
- [ ] PR reviewed and approved
- [ ] Merged to main
```
