# Scan Enhancements Implementation Summary

## Overview

This implementation adds media grouping functionality to Rosey as specified in `docs/SCAN_ENHANCEMENTS.md`. The system now groups files by media directory and classifies them as movies, shows, or unknown, enabling more accurate identification and organization.

## Changes Made

### 1. New Module: `rosey.grouper`

Created a new grouper module that handles post-scan media organization:

**Files:**
- `src/rosey/grouper/__init__.py` - Module exports
- `src/rosey/grouper/grouper.py` - Core grouping logic

**Key Components:**

#### `MediaGroup` Class
Represents a media directory with its files:
- `directory`: Absolute path to the media directory
- `kind`: Classification as "movie", "show", or "unknown"
- `primary_videos`: List of video files
- `companions`: Dict mapping video base names to companion files (subtitles, NFOs, etc.)
- `directory_companions`: Directory-level companions (e.g., movie.nfo, poster.jpg)
- `nfo_data`: Parsed directory-level NFO data
- `errors`: List of classification errors

#### Key Functions

**`get_media_directory(video_path, root_path)`**
- Determines the nearest qualifying ancestor folder as the media directory
- Skips generic organizational roots (source, tv, movies, media, downloads, etc.)
- Handles season folders by returning the parent show directory

**`build_media_groups(video_files, root_path, enforce_one_media)`**
- Groups videos by their media directory
- Discovers companion files (subtitles, NFOs, artwork)
- Parses directory-level NFO files
- Classifies each group

**`classify_group(group, enforce_one_media)`**
- Classifies media groups using multiple signals:
  - Season folder presence
  - Episode patterns in filenames (SxxExx, date-based)
  - NFO season/episode information
  - Single video → likely movie
  - Multiple videos → heuristic analysis
- Handles enforcement mode for mixed-content folders

**`_discover_companions(group)`**
- Matches companion files to primary videos by base name
- Includes files in permitted nested folders (Season XX, Subs, Extras)
- Identifies directory-level companions

### 2. Configuration Updates

**File:** `src/rosey/config/__init__.py`

Added new configuration option to `ScanningConfig`:
```python
enforce_one_media_per_folder: bool = False
```

When enabled, mixed-content folders are flagged as errors rather than attempting heuristic classification.

### 3. UI Integration

**File:** `src/rosey/ui/main_window.py`

#### ScanWorker Updates
- Modified to use `build_media_groups()` after scanning
- Processes files at the group level
- Passes group references with each item

#### MainWindow Updates
- Added `self.groups` dict to track media directories
- Modified `update_tree()` to display media directories instead of title-based nodes
- Updated tree nodes to show directory names (e.g., "Breaking Bad", "The Matrix (1999)")
- Modified `on_tree_selection()` to filter by media directory
- Updated `on_identify()` to apply identification to all items in a media group

#### Tree Structure
```
Movies/
├── The Matrix (1999)/
└── Inception (2010)/

Shows/
├── Breaking Bad/
└── The Office/

Unknown/
└── Mixed Content Folder/
```

Selecting a media node in the tree now shows all files discovered for that directory in the grid.

### 4. Tests

**File:** `tests/unit/test_grouper.py`

Created comprehensive test coverage (15 tests):
- Media directory detection for movies and shows
- Group building with companions
- Classification logic for various scenarios
- Mixed content handling (strict vs. relaxed)
- Generic root filtering
- Date-based episode recognition
- Multiple groups in single scan

All tests pass successfully.

## Design Decisions

### 1. Backward Compatibility
- Existing Scanner API unchanged
- Grouping is a post-scan layer
- Identifier, Planner, and Mover remain file-oriented
- Minimal changes to existing code paths

### 2. UI Changes
- Tree now shows media directories instead of titles
- More intuitive for users managing folder-based collections
- Right-click "Identify..." on a media node updates all contained items

### 3. Companion File Discovery
- Matches by base name to primary videos
- Supports nested organizational folders
- Recognizes directory-level assets (poster.jpg, movie.nfo)

### 4. Classification Heuristics
Priority order:
1. Season folders or episode patterns → Show
2. Single video + NFO with IDs (no season) → Movie
3. Multiple videos → Heuristic analysis or Unknown (based on enforcement)

### 5. Generic Root Filtering
Excluded folder names that don't qualify as media directories:
- source, sources, tv, movies, media
- downloads, video, videos
- incoming, complete

This ensures grouping happens at the meaningful media level.

## Acceptance Criteria (from spec)

✅ **Movie single-folder**: A movie directory with video and companions shows as one media node under Movies

✅ **Show with seasons**: Episodes across season folders group under one show node

✅ **Mixed-content folder**: Appears under Unknown (or errors with enforcement on)

✅ **Move preserves companions**: Existing mover already handles sidecars; no changes needed

## Edge Cases Handled

✅ Date-based episodes (YYYY-MM-DD) recognized as shows

✅ Multi-episode files supported by existing identifier/planner

✅ Multi-part movies handled by existing code

✅ Symlinks respected per follow_symlinks config

✅ Non-video files outside companion patterns ignored

## Testing

All quality checks pass:
- ✅ ruff check
- ✅ ruff format
- ✅ mypy type checking
- ✅ 641 pytest tests pass (including 15 new grouper tests)
- ✅ UI smoke test

## Future Enhancements (Not Implemented)

Per the spec's "Non-goals" section, these remain for future work:
- Deep content-based fingerprinting across directories
- Automatic splitting of mixed folders
- `enforce_one_media_per_folder` UI configuration
- Specials handling (Season 00)
- Provider-based episode title enrichment per group

## Files Modified

**New Files:**
- `src/rosey/grouper/__init__.py`
- `src/rosey/grouper/grouper.py`
- `tests/unit/test_grouper.py`

**Modified Files:**
- `src/rosey/config/__init__.py` (added enforce_one_media_per_folder)
- `src/rosey/ui/main_window.py` (integrated grouping into scan workflow and UI)

## Code Quality

All code follows project standards:
- Type hints throughout
- Comprehensive docstrings
- PEP 8 compliant (via ruff)
- mypy strict mode compliant
- Full test coverage for new functionality
