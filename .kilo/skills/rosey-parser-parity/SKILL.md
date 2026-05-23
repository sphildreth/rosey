---
name: rosey-parser-parity
description: Use when porting Python filename, episode, season, year, date, part, title cleanup, and TMDB path parsing into Rust.
---


# Rosey Parser Parity Skill

Use this skill for `patterns.py` and parser-related behavior.

## Python Reference

Read:

```text
../rosey/src/rosey/identifier/patterns.py
../rosey/src/rosey/identifier/identifier.py
../rosey/tests/
```

## Rust Target

Implement in:

```text
crates/rosey-core/src/patterns.rs
crates/rosey-core/tests/patterns_tests.rs
```

## Functions

Implement or update:

```rust
extract_title_before_episode(filename: &str) -> String
extract_episode_info(filename: &str, known_season: Option<u16>) -> Option<EpisodeMatch>
extract_date(filename: &str) -> Option<DateMatch>
extract_year(filename: &str) -> Option<u16>
extract_part(filename: &str) -> Option<u16>
extract_season_from_folder(folder_name: &str) -> Option<u16>
clean_title(raw: &str) -> String
extract_tmdb_id_from_path(path: &str) -> Option<String>
```

## Required Test Coverage

Cover:

```text
S01E02
S01 E02
S01E01-E02
1x02
1x02-03
S01EP02
Season 1 EP01
Episode 13 with known season
episode number at filename start with known season
dash-separated season/episode with known season
YYYY-MM-DD and YYYY.MM.DD daily show dates
year extraction that ignores daily-show dates
part/pt/Roman numeral/spelled-out part detection if Python supports it
Season 01 / Season 1 / S03 folder detection
common media artifact title cleanup
```

## Rules

- Preserve Python quirks during parity.
- Do not improve behavior silently.
- Add gaps to `/design/migration/parser-parity-notes.md`.
