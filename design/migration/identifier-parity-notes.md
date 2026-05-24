# Identifier Parity Notes

## Status

Implemented for the Python `Identifier` behavior used by CLI and TUI workflows. Rust now identifies episodes, movies, NFO-backed items, companion files, directory-constrained movies, duration-constrained movies, and provider-confirmed path TMDB IDs through `crates/rosey-core/src/identifier.rs` and `crates/rosey-metadata/src/identifier.rs`.

## Ported Behavior

- NFO discovery and parsing with parse errors preserved in `IdentificationResult.errors`.
- Episode detection from filename, season folders, date-style episodes, and NFO season/episode fields.
- Show-title derivation from season folder structure, non-generic parent folders, or filename fallback.
- Movie detection from filename years, folder years, multipart markers, NFO titles, and NFO movie IDs.
- `movies_always_in_own_directory` checks for season folders, show roots, and sibling media files.
- `minimum_movie_duration_minutes` checks through `ffprobe`, with fast-path skipping for TUI planning.
- Identifier-level companion discovery from same-directory images/subtitles and subtitle subdirectories.
- Path `[tmdbid-*]` extraction without treating the ID as confirmed metadata unless provider lookup succeeds.
- Provider-backed movie confirmation from TMDB/cache when online providers are configured.

## Intentional Deviations

- Rust has no long-lived `Identifier` object cache yet. Directory and duration checks are still cheap enough for current CLI use, and the TUI skips duration checks during planning.
- Provider confirmation is async and lives in `rosey-metadata`; offline identification stays in `rosey-core`.
- TUI provider search is terminal-native (`F5` in the identify overlay) rather than a PySide6 list dialog.

## Test Coverage

- Filename, parser, NFO, companion, and planner tests cover the pure behavior.
- `identifier_tests.rs` covers file-stem parsing, unconfirmed path TMDB IDs, duration rejection, and NFO movie IDs.
- `rosey-metadata` identifier tests cover disabled-provider behavior and cache-backed provider confirmation without network access.
