# Tests Guide

This project uses pytest with a CSV-driven strategy for filename/identifier coverage and focused unit tests for core modules.

## Structure

- tests/identifier/
  - fixtures/
    - filenames_tv.csv — TV episode cases (filename/path → expected fields)
    - filenames_movies.csv — Movie cases (filename/path → expected fields)
  - test_filenames_tv.py — Runs TV CSV cases via a shared harness
  - test_filenames_movies.py — Runs movie CSV cases via a shared harness
- tests/unit/ — Focused unit tests for patterns, planner, scanner, scorer, models, config, and NFO parsing.

Some legacy per-case tests were consolidated into the CSV approach to reduce duplication and brittleness. Modules kept in `tests/unit/` target stable APIs and behavior.

## Adding a CSV case

1. Pick the appropriate CSV in `tests/identifier/fixtures/`:
   - TV: `filenames_tv.csv`
   - Movies: `filenames_movies.csv`
2. Add a new row with the following columns:

   - source_path: Path or filename to parse (quotes recommended when containing spaces/commas)
   - kind: `episode` or `movie`
   - title: Expected cleaned title
   - year: Expected year (integer) or blank
   - season: Expected season number (integer) or blank
   - episodes: For TV, a single number or a pipe-separated list: `1` or `1|2`
   - part: Multipart number (e.g., `1`) or blank
   - date: For date-based episodes, `YYYY-MM-DD`, else blank
   - reason_contains: Optional substring to look for in parser reasons (kept blank by default to reduce brittleness)

3. Save the file and run tests:

```bash
pytest -q
```

## Expectations and notes

- Title cleaning removes release tags (WEB-DL, TVRip, Bluray, codecs, audio info), ranges (e.g., `Seasons 1 to 5`), episode patterns, dates, and years, then normalizes separators.
- Episodes inherit `year` from the show folder when present (e.g., `Good Times (1974)` → `year=1974`) or from NFO if available.
- For daily shows (date-based), `season`/`episodes` may be empty while `date` is populated.
- The CSV harness adapts to either a low-level parser (`parse_path`/`parse_media`/`parse_filename`) or falls back to `rosey.identifier.identify_file()` and normalizes its result.
- Avoid asserting on specific “reasons” text unless necessary; reason strings are diagnostic and may evolve.

## Writing focused unit tests

- Use `tests/unit/test_patterns.py` to add precise tests for pattern helpers (episode/date/year/part/title cleaning).
- Use `tests/unit/test_planner.py` for destination path formats, sanitization, and edge cases (unknown items, specials, multipart, multi-episode).
- Use `tests/unit/test_tmdb_parsing.py` and `tests/unit/test_identifier.py` for NFO precedence and unknown/edge cases.

## Running the suite

```bash
pytest -q
```

The VS Code Testing view is configured via `.vscode/settings.json` to discover and run pytest. Ensure your `.env` has `PYTHONPATH=src` so imports work in both tests and debugging sessions.
