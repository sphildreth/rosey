# Metadata Provider Parity Notes

## Status
Complete for the provider, cache, and application integration surface used by the Rust CLI/TUI. Python `TMDBProvider`, `TVDBProvider`, `ProviderCache`, and `ProviderManager` are ported to `crates/rosey-metadata`, `identify_file_with_metadata()` is wired into the CLI/TUI identify/plan flow for embedded `[tmdbid-*]` paths, and the TUI identify overlay can search TMDB manually when online providers are configured.

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
- **Rate limiting**: `MAX_RPS = 2` constant mirrors Python; TMDB has an active limiter and TVDB follows the Python provider shape
- **Endpoints**: `search_movie`, `search_tv`, `get_movie_by_id`, `get_tv_by_id`, `get_episode`
- **Episode lookup**: Fetches all episodes via `/series/{id}/episodes/default` and filters by season/episode number

## Provider Cache

- **Backend**: DecentDB via the `decentdb` crate, with Python-style `cache/provider_cache.ddb` directory handling
- **Schema**: `(provider, kind, key)` primary key; `data` (JSON text); `updated_at` (unix timestamp)
- **TTL**: Configurable in days; expired entries deleted on read
- **Operations**: `get`, `set`, `clear_expired`, `clear_all`, `stats`
- **Parameter binding**: Uses `$1`, `$2`, etc. positional parameters for safety (no string interpolation for user-controlled values)
- **Format updates**: Cache open retries unsupported DecentDB file formats through `decentdb-migrate`, writing a temporary `.migrating` file, preserving a `.backup`, and reopening the migrated database

## Provider Manager

- **Graceful degradation**: Returns empty results/`None` if providers disabled or not configured
- **Cache integration**: All `search_*` and `get_*` methods check cache before querying API, write back on success
- **Configuration**: `configure_tmdb(api_key, language, region)`, `configure_tvdb(api_key, language)`

## Intentional Deviations / Release Validation

1. **Automatic lookup matches identifier behavior**: batch identification enriches files that already carry a `[tmdbid-*]` path tag, matching Python `Identifier`. Title search remains an explicit TUI identify action.
2. **TVDB rate limiter shape**: `TvdbProvider` carries the Python `MAX_RPS = 2` constant but does not add a separate limiter because the Python implementation does not either. TMDB uses `RateLimiter`.
3. **TVDB `language` field unused**: Stored for API parity but not sent in requests, matching Python.
4. **Live online tests are release validation**: TMDB/TVDB providers are not unit-tested against live APIs by default because they require API keys and network. Cache and cache-backed provider confirmation are tested locally.
5. **No `close()` method**: Rust `reqwest::Client` is `Clone`/`Arc` internally; no explicit close needed.

## Dependencies Added

- `reqwest = { version = "0.13", features = ["json", "query"] }` — HTTP client
- `decentdb` pinned to `v2.7.0` — embedded cache database
- `tokio` (already present) — async runtime

## Test Coverage

- 8 cache tests in `crates/rosey-metadata/tests/cache_tests.rs`:
  - `cache_open_creates_schema`
  - `cache_open_accepts_python_style_directory`
  - `cache_set_and_get`
  - `cache_get_missing_returns_none`
  - `cache_expired_entry_returns_none`
  - `cache_clear_all`
  - `cache_clear_expired_returns_removed_count`
  - `cache_stats_counts_entries`

All cache tests use `tempfile::tempdir()` — no real user paths touched.

- Identifier provider tests in `crates/rosey-metadata/src/identifier.rs`:
  - disabled providers keep path TMDB IDs unconfirmed
  - cache-backed TMDB movie confirmation updates item metadata and confidence without network access

## Quality Gates

All passing:
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recommended Next Step

Add optional live-provider smoke tests gated on `TMDB_API_KEY` / `TVDB_API_KEY` if release validation needs to cover network behavior.

## DecentDB Update Process

- Bump the workspace `decentdb` Git tag when Rosey should track a new DecentDB release.
- Keep `decentdb-migrate` available in `PATH`, or point `ROSEY_DECENTDB_MIGRATE` at the matching tool binary.
- On `UnsupportedFormatVersion`, Rosey attempts an in-place update workflow using `provider_cache.ddb.migrating` and `provider_cache.ddb.backup`, then retries opening the cache.
- For disposable beta cache resets, `ROSEY_DECENTDB_RESET_ON_UNSUPPORTED=1` removes the unsupported cache file and recreates it instead of running the update tool.

## DecentDB cache file naming for setup phase

- Rename disposable cache filenames to `provider_cache.ddb` and `cache.ddb`.
- Remove references to `.db` names in phase-specific documentation and test expectations for provider cache files.
- No old cache data migration is required for this phase; fresh disposable cache files are acceptable.
