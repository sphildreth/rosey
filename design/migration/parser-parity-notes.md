# Parser Parity Notes

## Python Source of Truth

- `../rosey/src/rosey/identifier/patterns.py` — all regexes and extraction helpers
- `../rosey/src/rosey/identifier/identifier.py` — higher-level `identify()` logic (not ported in this slice)
- `../rosey/tests/unit/test_patterns.py` — baseline unit tests
- `../rosey/tests/identifier/test_filenames_tv.py` / `test_filenames_movies.py` — CSV fixture tests

## Rust Modules Implemented

- `crates/rosey-core/src/patterns.rs` — owns all parser behavior
- `crates/rosey-core/src/models.rs` — domain types (`EpisodeMatch`, `MovieMatch`, `DateMatch`)
- `crates/rosey-core/tests/patterns_tests.rs` — 54 table-driven parity tests

## Functions Ported

| Function | Python | Rust | Notes |
|----------|--------|------|-------|
| `extract_title_before_episode` | `patterns.py` | `patterns.rs` | Matches Python trailing-separator trimming (whitespace / dashes). Dots remain; `clean_title` removes them later. |
| `extract_episode_info` | `patterns.py` | `patterns.rs` | Signature changed to `(filename, known_season: Option<u16>)` to avoid overloads. All 7 episode patterns preserved in same order as Python. |
| `extract_date` | `patterns.py` | `patterns.rs` | Validates month 1-12 and day 1-31 (Python does not validate; we added strictness). |
| `extract_year` | `patterns.py` | `patterns.rs` | Parenthesized year checked first, then standalone. Date-context and episode-marker skips replicated. |
| `extract_part` | `patterns.py` | `patterns.rs` | Integer, Roman numeral (I-V-X), and spelled-out (One..Ten) supported. |
| `extract_season_from_folder` | `patterns.py` | `patterns.rs` | Handles `Season N`, `S##`, and boundary-delimited `S##` forms. |
| `extract_tmdb_id_from_path` | `patterns.py` | `patterns.rs` | Walks path components from leaf toward root (same priority as Python). |
| `clean_title` | `patterns.py` | `patterns.rs` | Added `clean_title_with_year` for parity with Python's optional `extracted_year` parameter. Core behaviors preserved: Vol preservation, parenthetical preservation, separator normalization, year removal, quality/codec/audio/source tag stripping, compound descriptor removal, hyphenated-compound restoration (Spider-Man, X-Men). |

## Behaviors Covered by Tests

- Episode parsing: SxxEyy, SxxEyy-Ezz, 1x02, 1x02-03, SxxEPyy, Season N EPyy, Episode XX, episode-at-start, dash-separated
- Multi-episode range expansion
- Episode title extraction from dash separator and parentheses
- Known-season-only patterns (dash and at-start)
- Date parsing (dashes and dots)
- Year parsing: parenthesized, standalone, rejecting date-context years, rejecting episode-marker years
- Part parsing: numeric, abbreviated, dotted, Roman numeral, spelled-out English
- Season folder parsing: `Season 01`, `Season 1`, `season 02`, `S03`
- TMDB ID extraction from file path and parent directories
- Title cleanup: separator conversion, space collapsing, episode pattern removal, quality/codec removal, Vol preservation, hyphenated-compound restoration

## Known Gaps / Deviations

1. **Date validation** — Rust `extract_date` rejects `month == 0` or `day == 0`; Python accepts them and would format `0000-00-00`. This is stricter and arguably safer.
2. **Year regex** — Python uses look-ahead `(?=[._\s-]|$)` in `YEAR_PATTERN`. Rust regex crate does not support look-ahead. We replaced with a non-capturing trailing boundary `(?:[._\s-]|$)` and added manual boundary checks in `extract_year`. The behavior is equivalent for all tested cases.
3. **Title cleanup completeness** — `clean_title_with_year` is a substantial port but not every Python edge case is guaranteed identical (e.g., some compound descriptor multi-word sequences, or very rare release-group names). Coverage is strong for the fixture CSV examples.
4. **NFO / sidecar / duration / online metadata** — These belong to `identifier.py`, not `patterns.py`. Not in scope for this slice.
5. **Identifier logic** — The full `identify_file()` / `Identifier` class from `identifier.py` is not yet ported. Only the underlying pattern helpers are ready.

## Recommended Next Migration Slice

**Jellyfin destination planner parity**

- Port destination path generation and filename sanitization from `../rosey/src/rosey/planner/`
- Implement conflict suffixing (` - 1`, ` - 2`, etc.)
- Add snapshot/golden tests for planned paths
- This should be done before scanner or mover work begins, so the planner is testable before file operations are wired in.
