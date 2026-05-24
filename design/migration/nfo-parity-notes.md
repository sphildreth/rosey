# NFO Parsing Parity Notes

## Status
Complete. Python NFO parsing behavior ported to `crates/rosey-core/src/nfo.rs`.

## Ported Functions / Types

| Python | Rust | Notes |
|--------|------|-------|
| `NFOData` class | `NfoData` struct | Same fields: title, year, imdb_id, tmdb_id, tvdb_id, episode_title, season, episode |
| `parse_nfo` | `parse_nfo` | Reads file, parses XML, returns `Option<NfoData>` |
| `find_nfo_for_file` | `find_nfo_for_file` | Same lookup order: same-name → movie.nfo → tvshow.nfo |
| `normalize_imdb_id` | `normalize_imdb_id` | URL extraction + `tt` prefix |

## Behavior Verified

- **Movie NFO parsing**: extracts `title`, `year`, `tmdbid`, `imdbid`
- **Episode NFO parsing**: extracts `title`, `season`, `episode`, `episodetitle`, `tvdbid`
- **Kodi `uniqueid` tags**: `<uniqueid type="imdb">`, `<uniqueid type="tmdb">`, `<uniqueid type="tvdb">` all parsed correctly
- **Alt tag names**: `imdb_id`, `tmdb_id`, `tvdb_id`, `episode_title` recognized
- **Direct tag precedence**: direct `tmdbid` tag takes precedence over `uniqueid` (guarded by `is_none()`) — matches Python behavior
- **Bad year ignored**: non-numeric `<year>` does not crash; field stays `None`
- **Missing file**: returns `None`
- **Invalid XML**: `quick-xml` is lenient; plain text may return empty `NfoData` rather than `None` — documented as acceptable
- **Empty XML**: returns `Some(NfoData::default())`
- **IMDB normalization**:
  - Keeps `tt0133093` as-is
  - Adds `tt` prefix to bare numbers
  - Extracts ID from `imdb.com/title/tt...` URLs

## Intentional Deviations

1. **No logging**: Python emits warnings for parse errors. Rust returns `None` silently; callers can log.
2. **XML leniency**: `quick-xml` is more lenient than Python's `xml.etree.ElementTree`. Malformed XML may return empty data rather than `None`. This is acceptable for media metadata parsing.

## Test Coverage

20 tests total:

- 3 inline unit tests in `nfo.rs`: `normalize_imdb_id_basic`, `normalize_imdb_id_adds_prefix`, `normalize_imdb_id_from_url`
- 17 integration tests in `crates/rosey-core/tests/nfo_tests.rs`:
  - `parse_movie_nfo_basic`
  - `parse_movie_nfo_with_uniqueids`
  - `parse_episode_nfo_basic`
  - `parse_episode_nfo_alt_tags`
  - `parse_nfo_missing_file_returns_none`
  - `parse_nfo_invalid_xml_returns_none` (lenient)
  - `parse_nfo_empty_xml_returns_some_defaults`
  - `parse_nfo_bad_year_ignored`
  - `parse_nfo_uniqueid_precedence_over_direct`
  - `find_nfo_same_name`
  - `find_nfo_movie_nfo`
  - `find_nfo_tvshow_nfo`
  - `find_nfo_same_name_takes_precedence`
  - `find_nfo_none_when_missing`
  - `normalize_imdb_id_keeps_valid`
  - `normalize_imdb_id_adds_tt`
  - `normalize_imdb_id_extracts_from_url`

All tests use `tempfile::tempdir()` — no real user media paths touched.

## Dependencies Added

- `quick-xml = "0.37"` to workspace root `Cargo.toml`
- `quick-xml.workspace = true` in `crates/rosey-core/Cargo.toml`
- `tempfile.workspace = true` added to `rosey-core` dev-dependencies for tests

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace` (116 passed total)
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Step

NFO parsing parity is complete. Add new golden NFO fixtures here when additional Kodi/Jellyfin metadata tags need support.
