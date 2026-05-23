# Planner Parity Notes

## Status
Complete. All Python planner behavior has been ported to `crates/rosey-core/src/planner.rs`.

## Ported Functions

| Python | Rust | Notes |
|--------|------|-------|
| `Planner.__init__` | `Planner { movies_root, tv_root }` | Same defaults (empty strings) |
| `Planner.plan_destination` | `Planner::plan_destination` | Routes by `MediaKind` |
| `Planner._plan_movie` | `Planner::plan_movie` | Private method |
| `Planner._plan_episode` | `Planner::plan_episode` | Private method |
| `title_case` | `title_case` | Preserves lowercase articles/prepositions |
| `sanitize_name` | `sanitize_name` | Cross-platform filename sanitization |
| `plan_path` | `plan_path` | Convenience function |

## Behavior Verified

- **Movies**: folder name `Title (Year) [tmdbid-ID]`, filename with optional `Part N`
- **Episodes**: folder name `Title (Year) [tmdbid-ID]`, season folder `Season NN`, filename with `SxxExx`, multi-episode ranges, date-based naming, episode titles, multipart episodes
- **Sanitization**: removes `< > : " / \ | ? *`, collapses spaces, trims spaces/dots, handles Windows reserved names, defaults empty to `"unknown"`
- **Title casing**: capitalizes first and last words, lowercases articles/prepositions in between
- **No root set**: returns `source_path` unchanged

## Known Gaps / Intentional Deviations

1. **Companion kind**: Python `planner.py` checks for `kind == "companion"` and returns `"Moved with primary video"`, but `rosey.models.MediaItem.kind` only documents `"movie" | "show" | "episode" | "unknown"`. The Rust `MediaKind` enum matches the documented Python kinds. The `companion` branch in Python appears to be dead code.
2. **Conflict suffixing**: Not implemented in Python planner either. The Python reference does not generate conflict-suffixed paths; this is handled elsewhere (if at all).
3. **`Show` kind handling**: Rust `MediaKind::Show` falls through to the default case and returns `source_path`. Python does not have explicit `show` handling in the planner (shows are typically represented as collections of episodes).

## Test Coverage

22 tests in `crates/rosey-core/tests/planner_tests.rs`:

- 5 sanitize tests
- 1 title_case test
- 5 movie planning tests (basic, no year, multipart, no root, tmdb id)
- 8 episode planning tests (basic, multi-episode, with title, date-based, multipart, specials, no root, fixture)
- 1 unknown handling test
- 1 convenience function test

Golden fixture: `episode_all_in_the_family_fixture` matches Python output exactly.

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
