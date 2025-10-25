
# Rosey — Product Requirements Document (PRD)
Version: 1.0 • Date: 2025-10-25

## Related documents
- Technical Specification: [TECH_SPEC.md](./TECH_SPEC.md)
- Documentation Index: [README.md](./README.md)
- UI Mockups: [mockups/UI_MOCKUPS.md](./mockups/UI_MOCKUPS.md)

## 1) Overview
Rosey is a cross‑platform desktop utility (Windows + Linux) that scans a **Source** folder (including network shares),
identifies Movies and TV Shows using **offline signals** (.nfo, filenames, folders) and optional **online metadata lookups**
(TMDB/TVDB), assigns a confidence (Green/Yellow/Red), and upon user approval moves selected items into Jellyfin‑friendly folders.

Focus: **simple**, **responsive**, **theme‑able** (light/dark), and **fast I/O**. All file operations are opt‑in and logged.
Privacy‑first: no telemetry; provider calls only when enabled.

## 2) Target users & JTBD
- Home media owners who want reliable file organization for Jellyfin.
- When folders are messy or ambiguous, Rosey should confidently identify items and batch move them safely.

## 3) Success metrics
- ≥95% of **Green** items scan correctly in Jellyfin without manual edits.
- First‑run setup under 60 seconds (pick folders → Scan).
- Batch move throughput: **atomic rename** on same volume; copy+delete across volumes with bounded concurrency.
- UI remains responsive during long scans/moves (no freezes).
- Clear, searchable session logs for every operation.

## 4) Scope (MVP)
### In scope
- Single window PySide6 UI (Qt Widgets) with **light/dark** theme toggle.
- Folder pickers: **Source**, **Movies Target**, **TV Target**.
- Scanning (recursive) of local + network shares (UNC on Windows; SMB/NFS mounts on Linux).
- Identification (offline): `.nfo`, folder names, filename patterns (`SxxEyy`, `1x02`, `YYYY-MM-DD`, multi-episode `S01E01-E02`, multi-part `Part 1`).
- Identification (online, optional): **TMDB** primary, **TVDB** optional; caching and rate limiting.
- Confidence scoring with short “reasons” tooltip (e.g., "Matched filename pattern", "Found TMDB ID in NFO").
- Tree view of identified Shows and Movies (On left side of window).
- Grid with checkboxes; **Select All Green**; **Move Selected** (On right side of window).
- Jellyfin naming defaults:
  - Movie: `Movies/Title (Year) [ExternalId]/Title (Year).ext`
  - TV: `Shows/Show Title (Year) [ExternalId]/Season 01/Show Title - S01E02 - Episode Title.ext`
  - Specials/Extras/Bonus → `Extras`.
- Conflict dialog (Skip / Replace / Keep Both); "Keep Both" appends a numeric suffix, e.g., `(1)`.
- Config saved to `rosey.json` (paths, provider keys, theme, concurrency, cache TTL).
- Logging to file (rotating) and on‑screen status pane.

### Out of scope (post‑MVP)
- Artwork/NFO generation.
- Daemon/scheduler mode.
- Advanced duplicate detection/merging.
- Multi‑user sync.
- DVD/Blu-ray folder structures (`VIDEO_TS`, `BDMV`).

## 5) UX requirements
- See [UI_MOCKUPS](./mockups/UI_MOCKUPS.md)
  - Toolbar provides confidence filters: All, Green, Yellow, Red.
  - Selecting a node in the Library Tree scopes the grid to that show/season or movies group.

## 6) Functional requirements
- **FR‑1 Scan:** recursively enumerate **Source** with concurrency tuned for local vs network shares.
  - AC: Scanning 50k files completes without UI freeze; permission errors appear in the log and do not crash the app.
- **FR‑2 Identify (offline):** classify Movie/TV/Unknown from `.nfo`, folders, and filenames, including multi-episode (`S01E01-E02`) and multi-part (`Part 1`) patterns.
  - AC: At least the listed patterns are parsed into season/episode(s)/part; malformed files fall back to Unknown with a clear reason.
- **FR‑3 Online Lookups (opt‑in):** TMDB/TVDB with local cache and rate limit; localization (language/region).
  - AC: When enabled, provider requests respect rate limits; failures degrade gracefully and are logged.
- **FR‑4 Score:** 0–100 with explicit reasons (e.g., "Matched filename", "Found ID in .nfo"); thresholds: Green ≥70, Yellow 40–69, Red <40.
  - AC: "Select All Green" checks only items with confidence ≥70; Yellow remain unchecked; Red never auto-selected.
- **FR‑5 Plan:** compute sanitized destination paths following Jellyfin rules, including for multi-episode/part files; preserve extension.
  - AC: Destination paths are valid on Windows/Linux; reserved names and invalid characters are sanitized consistently.
  - Multi-episode naming: `Show Title - S01E01-E02 - Episode Title.ext`.
  - Multipart naming (same episode): `Show Title - S02E03 Part 1.ext`, `... Part 2.ext`.
  - Specials: use `Season 00`.
- **FR‑6 Present:** show candidates; allow selection helpers; filter and sort.
  - AC: Confidence filter buttons (All/Green/Yellow/Red) modify the grid; Unknown items are visible but excluded from plan by default.
- **FR‑7 Move:** execute file operations with robust error handling and progress updates. On same volume, use atomic renames. Across volumes, use a safe copy-verify-quarantine process (copy, verify size, then move original to a temporary folder). Moves should be transactional; if any part of a multi-file operation fails, the system will attempt to roll back any changes made in that operation to leave the destination clean.
- **FR-7.1 Move Sidecar Files:** When a media file is moved, all associated sidecar files (e.g., `.srt`, `.nfo`, `.jpg`) sharing the same base filename are also moved.
  - Preflight checks: verify free space on destination (≥ size of files to copy + small buffer), check path length and permissions; surface actionable errors.
- **FR‑8 Config & Logging:** load/save `rosey.json`; rotate logs; redact secrets in logs.
- **FR‑9 Theming:** global light/dark toggle; remember user’s last theme.
- **FR‑10 Dry Run:** Preview mode available in config and as a command-line flag; simulates moves without file operations, logging planned actions.

### Supported formats & sidecars
- Video: mkv, mp4, mov, avi, ts (extensible).
- Subtitles/sidecars: srt, ass, vtt, nfo, jpg, png (moved with matching base filename).
- Unknowns: files not matching supported media or patterns are shown as Unknown and excluded by default.

## 7) Non‑functional requirements
- **Platforms:** Windows 10/11 x64; modern Linux x64.
- **Performance:** UI thread never blocked; scan 50k files without UI stutter; moves respect bounded concurrency (local>network).
- **Reliability:** retries on transient I/O; safe shutdown; no destructive operations without user action. For network shares, handle disconnections during moves by pausing and retrying with exponential backoff; notify user if retries fail.
- **Packaging:** PyInstaller binaries for Win/Linux; ship icons for installers (ICO/PNG).

## 8) Risks & mitigations
- Provider rate limits → caching + exponential backoff; offline continues.
- Network share slowness → reduced concurrency + retries; clear progress feedback.
- Misidentification → show reasons; keep Yellow unchecked by default; dry‑run preview (optional flag).
- Long paths on Windows → normalization and error guidance.

## 9) Release plan
- **M1:** Scanner + offline Identify + Score + Plan + UI table (no moves).
- **M2:** Mover + conflict handling + logging + progress/cancel.
- **M3:** Online lookups + cache + settings UI.
- **M4:** Packaging, icons, docs, first binary.

### Appendices

#### Shows Jellyfin Folder Organization Example
```
Shows
├── Series Name A (2010)
│   ├── Extras
│   │   ├── Some Special.mkv
│   │   ├── Interview with the Director.mp4
│   │   ├── 10 yr Anniversary Reunion Special.mp4
│   ├── Season 00
│   │   ├── Series Name A S00E01.mkv
│   │   └── Series Name A S00E02.mkv
│   ├── Season 01
│   │   ├── Series Name A S01E01-E02.mkv
│   │   ├── Series Name A S01E03.mkv
│   │   └── Series Name A S01E04.mkv
│   │   └── Series Name A S01E04.ja.ass
│   └── Season 02
│       ├── Series Name A S02E01.mkv
│       ├── Series Name A S02E02.mkv
│       ├── Series Name A S02E03 Part 1.mkv
│       └── Series Name A S02E03 Part 2.mkv
└── Series Name B (2018) [tmdbid-39]
    ├── Season 01
    |   ├── Series Name B S01E01.mkv
    |   └── Series Name B S01E02.mkv
    └── Season 02
        ├── Series Name B S02E01-E02.mkv
        └── Series Name B S02E03.mkv
```

#### Movies Jellyfin Folder Organization Example
```
Movies
├── Best_Movie_Ever (2019)
│   ├── Best_Movie_Ever (2019).mp4
│   ├── Best_Movie_Ever (2019).en_us.srt
└── Movie (2021) [imdbid-tt12801262]
│   ├── Movie (2021) [imdbid-tt12801262].mp4    
```

#### References
- [Jellyfin Shows Documentation](https://jellyfin.org/docs/general/server/media/shows/)
- [Jellyfin Movies Documentation](https://jellyfin.org/docs/general/server/media/movies)

### Glossary
- Green/Yellow/Red: Confidence buckets derived from score thresholds (≥70, 40–69, <40).
- Sidecar: Non-video companion file sharing the same base filename (e.g., `.srt`, `.nfo`, artwork).
- Transactional move: Grouped file operation that rolls back partial copies on failure.
- Cross-volume: Source and destination are on different volumes; requires copy/verify/remove instead of atomic rename.