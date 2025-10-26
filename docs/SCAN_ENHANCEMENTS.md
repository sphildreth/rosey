# Scan Enhancements

Refine how scanning is performed and add the requirement that shows and movies be in their own subfolders for accurate and efficient identification and processing.

## Goal

Quickly and efficiently identify shows and movies and semantically group files that belong together, enabling confident planning and moving while preserving companion files.

## Definitions

- Media directory: a folder that directly contains the primary media for a single work (movie) or a single show entity (a show root with seasons/episodes). Media directories may include nested organizational subfolders such as `Season 01`, `Subs`, `Extras`.
- Media node: a UI tree node representing a single media directory. Selecting a media node shows all files discovered for that directory in the grid.
- Primary video: a video file with a recognized extension (see `rosey.scanner.VIDEO_EXTENSIONS`).
- Companion (aka sidecar in code): non-video file associated with a primary video (e.g., `.nfo`, `.srt`, `.ass`, `.sub/.idx`, `.jpg/.png`). Companions move with their primary video. Note: the code and config currently use the term "sidecar" (e.g., `SIDECAR_EXTENSIONS`); this document prefers "companion" for readability but maps to the same concept.
- Show root: the top folder for a show which may contain season folders and episodes. `Season 00` is reserved for Specials.

## Assumptions and configuration

- enforce_one_media_per_folder (new):
  - true: mixed-content folders are flagged as errors/warnings; auto-grouping across directories is disabled.
  - false: heuristic grouping proceeds but warnings are surfaced for potential issues.
- scanning.follow_symlinks: respected from config.
- scanning.concurrency_local: used by the existing file scanner.

## Process flow

1) Scan files (existing)
- Use the existing `Scanner` to enumerate files under the configured Source folder(s). No changes to the scanner API or tests are required.

2) Build media groups (new logic after scan)
- Group by media directory: For each primary video file, choose the nearest qualifying ancestor folder as its media directory.
  - Qualifying ancestors exclude generic organizational roots such as `source`, `tv`, `movies`, `media`, `downloads`.
  - Include permitted nested folders inside a media directory: `Season XX`, `Subs`, `Subtitles`, `Extras`.
- Collect for each group:
  - primary_videos: all video files under the media directory (including allowed nested folders).
  - companions: files with companion/sidecar extensions that match the base name of a primary video (or common artwork/NFO files at the directory level).
  - nfo: any NFO parsed at directory level and per-file (reusing `identifier.nfo`).

3) Classify group (movie | show | unknown)
- Inputs: directory name, file names, presence of `Season XX` folders, NFO contents, counts of primary videos.
- Rules (applied in order):
  - If season folders exist or episode-pattern filenames (`SxxExx`, date-based) are found → Show.
  - If a single top-level primary video exists and NFO lacks season info → Movie.
  - If NFO includes `season`/`episode` → Show; if NFO includes only TMDB/IMDB and no season → Movie.
  - Otherwise → Unknown. If `enforce_one_media_per_folder=true`, surface as error and require fix/split.

4) Identify
- For Movies: identify the single primary video; title/year from filename or NFO; look up TMDB if enabled.
- For Shows: identify the show at the group level (title/year/tmdbid). Optionally enrich per-episode titles via providers when `season` and `episode` are known.
- Reuse existing `Identifier` per-file; apply group-level results to contained items as appropriate.

5) UI behavior
- Tree: top-level nodes are `Movies`, `Shows`, `Unknown`.
- Under each, add media nodes representing media directories (directory name as label).
- Left-click media node: the grid shows all files (primary videos and companions) discovered for that media directory.
- Right-click media node: `Identify…` opens the Identify dialog. On OK, apply identification to the entire group, re-score, and re-plan destinations.

6) Plan and move
- Use existing planner to compute destinations per identified item, based on `paths.movies` and `paths.tv`.
- Use existing mover to move the selected items, ensuring companions (sidecars) follow their primary video. Honor conflict policy and dry-run settings.

7) Error handling and logging
- Permission and IO errors are logged in the activity log and attached to affected media groups.
- Ambiguous or mixed-content media directories are surfaced under `Unknown` with a note to fix or split folders (or relax enforcement).

## Data model (doc-level)

Introduce a conceptual MediaGroup (no hard API change required to scanner):

- id: absolute path of the media directory
- kind: `movie` | `show` | `unknown`
- files.primary: list of primary video files
- files.companions: mapping of primary base name → list of companion files; plus directory-level companions
- nfo.group: directory-level NFO-derived fields (title/year/tmdbid/tvdbid)
- nfo.items: per-file NFO data where present

Implementation note: grouping occurs after file scan; the existing `Scanner` stays file-oriented and unchanged.

## Acceptance criteria

- Movie single-folder
  - Given `Movie (2010)/Movie (2010).mkv` with `Movie (2010).srt` and `movie.nfo`, one media node appears under `Movies` showing one primary video and related companions. Identify sets title/year/tmdbid; planned path is `Movies/Movie (2010)/Movie (2010).mkv`.
- Show with seasons
  - Given `Show (2012)/Season 01/S01E01.mkv` and `Season 01/S01E02.mkv`, one media node appears under `Shows`. Identify sets the show TMDB id; episode titles populate when available; planned paths follow `Show/Season 01/Show - S01E01.ext`.
- Mixed-content folder
  - Given a folder containing two unrelated movies, the folder appears under `Unknown` (or errors if enforcement is on) and cannot be identified as a single entity until split.
- Move preserves companions
  - When moving selected items, companions (sidecars) with matching base names move alongside the primary video.

## Edge cases

- Date-based episodes (YYYY-MM-DD) recognized as Show.
- Multi-episode files (`S01E01E02`) planned with range format `S01E01-E02`.
- Multi-part movies detected via `Part N` suffix.
- Specials go to `Extras` when season is unknown or marked special.
- Symlinks are included/excluded per `follow_symlinks` config.
- Non-video files outside recognized companion/sidecar patterns are ignored for planning/moving.

## Testing plan

- Grouping/classification unit tests:
  - `movie_single_file_with_companions`
  - `show_with_seasons_and_episodes`
  - `mixed_folder_conflict`
  - `unknown_folder_requires_identify`
- UI integration behavior:
  - Selecting a media node filters the grid to that group; right-click Identify updates all contained items and re-plans destinations.
- Backward compatibility:
  - Existing scanner tests remain valid; grouping is a post-scan layer.

## Non-goals (for now)

- Deep content-based fingerprinting to merge across directories.
- Automatic splitting of mixed folders; we surface them for user action instead.

## Notes on alignment with current code

- Scanner (`rosey.scanner.Scanner`) remains file-based; grouping happens after scan.
- Identifier already infers movie vs episode from filenames/NFO; group-level identification aggregates those signals.
- Planner/Mover are reused as-is; destinations are still computed per `MediaItem`.

## Examples

### Show

This show is named "Maude" and has 6 seasons and an "Extras" directory

```
Maude/
├── Maude - Season 1
│   ├── Maude - S01E01 - Maude's Problem (a.k.a.) Maude and the Psychiatrist.mp4
├── Maude - Season 2
│   ├── Maude - S02E04 - Maude's Facelift (1).mp4
│   ├── Maude - S02E05 - Maude's Facelift (2).mp4
├── Maude - Season 3
├── Maude - Season 4
├── Maude - Season 5
├── Maude - Season 6
└── Maude - Specials
```
