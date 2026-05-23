# Metadata Provider Parity Notes

## Status
Complete. Python `TMDBProvider`, `TVDBProvider`, `ProviderCache`, and `ProviderManager` ported to `crates/rosey-metadata`.

## Ported Components

| Python | Rust | File |
|--------|------|------|
| `TMDBProvider` | `TmdbProvider` | `src/providers.rs` |
| `TVDBProvider` | `TvdbProvider` | `src/providers.rs` |
| `RateLimiter` | `RateLimiter` | `src/providers.rs` |
| `ProviderCache` | `ProviderCache` | `src/cache.rs` |
| `ProviderManager` | `ProviderManager` | `src/manager.rs` |

## TMDB Provider

- **Rate limiting**: Token-bucket limiter at 4 req/s (conservative vs TMDB's ~40/10s)
- **Endpoints**: `search_movie`, `search_tv`, `get_movie_by_id`, `get_tv_by_id`, `get_episode`
- **Retry on 429**: 2-second backoff, single retry
- **Error handling**: Graceful degradation — returns `None`/`Vec::new()` on any HTTP/parse error

## TVDB Provider

- **Authentication**: JWT token via `/login` endpoint, cached with 30-day expiry (minus 1h buffer)
- **Rate limiting**: `MAX_RPS = 2` constant present (not yet wired; uses same `RateLimiter` pattern as TMDB)
- **Endpoints**: `search_movie`, `search_tv`, `get_movie_by_id`, `get_tv_by_id`, `get_episode`
- **Episode lookup**: Fetches all episodes via `/series/{id}/episodes/default` and filters by season/episode number

## Provider Cache

- **Backend**: SQLite via `sqlite` crate
- **Schema**: `(provider, kind, key)` primary key; `data` (JSON text); `updated_at` (unix timestamp)
- **TTL**: Configurable in days; expired entries deleted on read
- **Operations**: `get`, `set`, `clear_expired`, `clear_all`, `stats`
- **Parameter binding**: Uses `?1`, `?2`, etc. positional parameters for safety (no string interpolation for user-controlled values)

## Provider Manager

- **Graceful degradation**: Returns empty results/`None` if providers disabled or not configured
- **Cache integration**: All `search_*` and `get_*` methods check cache before querying API, write back on success
- **Configuration**: `configure_tmdb(api_key, language, region)`, `configure_tvdb(api_key, language)`

## Known Gaps / Intentional Deviations

1. **No TVDB rate limiter wired**: `TvdbProvider` has `MAX_RPS = 2` constant but does not use `RateLimiter` yet. TMDB uses `RateLimiter`.
2. **TVDB `language` field unused**: Stored for API parity but not sent in requests.
3. **No online tests**: TMDB/TVDB providers are not unit-tested against real APIs (requires API keys and network). Cache is tested with temp SQLite DBs.
4. **No `close()` method**: Rust `reqwest::Client` is `Clone`/`Arc` internally; no explicit close needed.
5. **Cache `clear_expired` rowcount**: Returns `0` because the `sqlite` crate does not expose `sqlite3_changes()` easily.

## Dependencies Added

- `reqwest = { version = "0.13", features = ["json", "query"] }` — HTTP client
- `sqlite = "0.36"` — SQLite bindings
- `tokio` (already present) — async runtime

## Test Coverage

- 6 cache tests in `crates/rosey-metadata/tests/cache_tests.rs`:
  - `cache_open_creates_schema`
  - `cache_set_and_get`
  - `cache_get_missing_returns_none`
  - `cache_expired_entry_returns_none`
  - `cache_clear_all`
  - `cache_stats_counts_entries`

All cache tests use `tempfile::tempdir()` — no real user paths touched.

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace` (152 passed total)
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Slice

**TUI implementation** (`rosey-tui` with Ratatui) — the core engine, CLI, and metadata providers are all now testable and complete. The TUI can be built on top of:
- `rosey-core` (parser, planner, models)
- `rosey-fs` (scanner, mover)
- `rosey-metadata` (providers, cache)
- `rosey-cli` (command structure, identification logic)

Alternatively, **journal/recovery** parity — implement the JSON Lines operation journal mentioned in `design/SPEC.md` for crash visibility and resume support.
