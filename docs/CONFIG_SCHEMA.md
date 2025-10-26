# Configuration Schema — Rosey

## Overview

Rosey uses a JSON configuration file (`rosey.json`) stored in the user's home directory or application data folder. The config file is optional; if not present, defaults are used.

### Config File Location

**Linux**: `~/.config/rosey/rosey.json`
**Windows**: `%APPDATA%\rosey\rosey.json`

## Schema Definition

```json
{
  "version": "1.0",
  "paths": {
    "source": "/path/to/media/source",
    "movies": "/path/to/Jellyfin/Movies",
    "tv": "/path/to/Jellyfin/Shows"
  },
  "ui": {
    "theme": "system",
    "window": {
      "width": 1200,
      "height": 800,
      "maximized": false
    },
    "splitters": {
      "main": [300, 900],
      "vertical": [600, 200]
    }
  },
  "behavior": {
    "dry_run": true,
    "auto_select_green": true,
    "conflict_policy": "ask"
  },
  "scanning": {
    "concurrency_local": 8,
    "concurrency_network": 2,
    "follow_symlinks": false,
    "extensions": {
      "video": [".mkv", ".mp4", ".mov", ".avi", ".ts", ".m4v"],
      "sidecar": [".srt", ".ass", ".vtt", ".nfo", ".jpg", ".png", ".xml"]
    }
  },
  "identification": {
    "use_online_providers": false,
    "confidence_thresholds": {
      "green": 70,
      "yellow": 40
    },
    "prefer_nfo_ids": true
  },
  "providers": {
    "tmdb": {
      "enabled": true,
      "api_key": "",
      "language": "en-US",
      "rate_limit_per_second": 4
    },
    "tvdb": {
      "enabled": false,
      "api_key": "",
      "language": "en"
    }
  },
  "caching": {
    "backend": "diskcache",
    "path": "~/.cache/rosey/metadata",
    "ttl_days": 30,
    "max_size_mb": 100
  },
  "logging": {
    "level": "INFO",
    "file_path": "~/.local/share/rosey/logs/rosey.log",
    "max_file_size_mb": 10,
    "backup_count": 5,
    "redact_secrets": true,
    "log_to_console": false
  },
  "move_operations": {
    "verify_copy": true,
    "quarantine_on_failure": true,
    "preserve_timestamps": true,
    "create_parent_dirs": true
  }
}
```

## Field Descriptions

### version
- Type: `string`
- Default: `"1.0"`
- Description: Config schema version for migration support

### paths
Configuration for source and destination directories.

- **source**: Path to scan for media files
- **movies**: Destination path for organized movies
- **tv**: Destination path for organized TV shows

All paths support:
- Absolute paths
- Tilde expansion (`~`)
- Environment variables (`$HOME`, `%APPDATA%`)
- UNC paths on Windows (`\\server\share`)

### ui
User interface preferences.

#### theme
- Type: `"system" | "light" | "dark"`
- Default: `"system"`
- Description: Color theme for the application

#### window
Window size and position (persisted between runs).

- **width**: Window width in pixels
- **height**: Window height in pixels
- **maximized**: Whether window is maximized

#### splitters
Splitter positions (persisted between runs).

- **main**: `[tree_width, details_width]`
- **vertical**: `[top_height, log_height]`

### behavior
Application behavior settings.

#### dry_run
- Type: `boolean`
- Default: `true`
- Description: If true, simulate moves without actually moving files

#### auto_select_green
- Type: `boolean`
- Default: `true`
- Description: Automatically check items with Green confidence on scan

#### conflict_policy
- Type: `"ask" | "skip" | "replace" | "keep_both"`
- Default: `"ask"`
- Description: How to handle destination file conflicts

### scanning
File scanning configuration.

#### concurrency_local
- Type: `integer`
- Default: `8`
- Description: Number of concurrent threads for local filesystem scans

#### concurrency_network
- Type: `integer`
- Default: `2`
- Description: Number of concurrent threads for network share scans

#### follow_symlinks
- Type: `boolean`
- Default: `false`
- Description: Whether to follow symbolic links during scanning

#### extensions
File extensions to recognize.

- **video**: Video file extensions to identify as media
- **companion (sidecar)**: File extensions for companion/sidecar files to co-move with media

Note: The JSON schema key remains `sidecar` for backward compatibility (e.g., `"sidecar": [".srt", ...]`). In documentation we refer to these as companion files for clarity.

### identification
Media identification settings.

#### use_online_providers
- Type: `boolean`
- Default: `false`
- Description: Enable online metadata lookups (TMDB/TVDB)

#### confidence_thresholds
Scoring thresholds for Green/Yellow/Red classification.

- **green**: Minimum confidence for Green (default: 70)
- **yellow**: Minimum confidence for Yellow (default: 40)
- Red is anything below yellow threshold

#### prefer_nfo_ids
- Type: `boolean`
- Default: `true`
- Description: Prefer IDs from `.nfo` files over filename matching

### providers
Online metadata provider configuration.

#### tmdb
TMDB (The Movie Database) configuration.

- **enabled**: Enable TMDB lookups
- **api_key**: API key (stored in OS keyring if available)
- **language**: Language code (e.g., `en-US`, `fr-FR`)
- **rate_limit_per_second**: Max requests per second

#### tvdb
TVDB (TheTVDB) configuration.

- **enabled**: Enable TVDB lookups
- **api_key**: API key (stored in OS keyring if available)
- **language**: Language code (e.g., `en`, `fr`)

**Security Note**: API keys in `rosey.json` are for development only. Production should use OS keyring (e.g., Windows Credential Manager, macOS Keychain, Secret Service on Linux).

### caching
Metadata cache configuration.

#### backend
- Type: `"diskcache" | "sqlite" | "none"`
- Default: `"diskcache"`
- Description: Cache backend to use

#### path
- Type: `string`
- Default: `"~/.cache/rosey/metadata"`
- Description: Cache directory path

#### ttl_days
- Type: `integer`
- Default: `30`
- Description: Cache entry time-to-live in days

#### max_size_mb
- Type: `integer`
- Default: `100`
- Description: Maximum cache size in megabytes

### logging
Logging configuration.

#### level
- Type: `"DEBUG" | "INFO" | "WARNING" | "ERROR" | "CRITICAL"`
- Default: `"INFO"`
- Description: Minimum log level to record

#### file_path
- Type: `string`
- Default: `"~/.local/share/rosey/logs/rosey.log"`
- Description: Log file path (rotating)

#### max_file_size_mb
- Type: `integer`
- Default: `10`
- Description: Maximum log file size before rotation

#### backup_count
- Type: `integer`
- Default: `5`
- Description: Number of rotated log files to keep

#### redact_secrets
- Type: `boolean`
- Default: `true`
- Description: Redact API keys and sensitive data in logs

#### log_to_console
- Type: `boolean`
- Default: `false`
- Description: Also log to console/stdout (useful for debugging)

### move_operations
File move operation settings.

#### verify_copy
- Type: `boolean`
- Default: `true`
- Description: Verify file size after cross-volume copy

#### quarantine_on_failure
- Type: `boolean`
- Default: `true`
- Description: Move source to quarantine folder if move fails

#### preserve_timestamps
- Type: `boolean`
- Default: `true`
- Description: Preserve modification times when copying

#### create_parent_dirs
- Type: `boolean`
- Default: `true`
- Description: Automatically create parent directories if missing

## Loading and Saving

### Pydantic Model

```python
from pydantic import BaseModel, Field
from typing import List, Literal

class PathsConfig(BaseModel):
    source: str = ""
    movies: str = ""
    tv: str = ""

class UIConfig(BaseModel):
    theme: Literal["system", "light", "dark"] = "system"
    window: dict = Field(default_factory=lambda: {"width": 1200, "height": 800, "maximized": False})
    splitters: dict = Field(default_factory=lambda: {"main": [300, 900], "vertical": [600, 200]})

class BehaviorConfig(BaseModel):
    dry_run: bool = True
    auto_select_green: bool = True
    conflict_policy: Literal["ask", "skip", "replace", "keep_both"] = "ask"

class RoseyConfig(BaseModel):
    version: str = "1.0"
    paths: PathsConfig = Field(default_factory=PathsConfig)
    ui: UIConfig = Field(default_factory=UIConfig)
    behavior: BehaviorConfig = Field(default_factory=BehaviorConfig)
    # ... other sections
```

### Config Manager

The `rosey.config` module provides:

- `load_config() -> RoseyConfig`: Load config from disk or defaults
- `save_config(config: RoseyConfig) -> None`: Save config to disk
- `get_config_path() -> Path`: Get platform-specific config path
- `migrate_config(old_version: str) -> None`: Migrate old config formats

## Migration Strategy

When the schema version changes:

1. Load existing config file
2. Check `version` field
3. Apply migrations sequentially (e.g., `1.0 -> 1.1 -> 1.2`)
4. Save migrated config
5. Log migration actions

Example migration:
```python
def migrate_1_0_to_1_1(config: dict) -> dict:
    # Add new field with default
    if "behavior" not in config:
        config["behavior"] = {"dry_run": True}
    config["version"] = "1.1"
    return config
```

## Validation

All config fields are validated on load:

- **Type checking**: Enforced by Pydantic
- **Path existence**: Warned if paths don't exist (not fatal)
- **API key format**: Validated if providers enabled
- **Numeric ranges**: Constrained (e.g., concurrency > 0)

Invalid config → log errors → fall back to defaults → allow user to fix via UI.

## Recording Strategy for Provider Fixtures

### Overview

To avoid hitting live APIs during CI runs and tests, we record provider responses as fixtures.

### Fixture Storage

- Location: `tests/fixtures/providers/`
- Format: JSON files named by request signature
- Structure:
  ```
  tests/fixtures/providers/
    tmdb/
      search_movie_the_matrix.json
      movie_603_en-US.json
    tvdb/
      search_series_the_office.json
      episodes_series_73244_season_2.json
  ```

### Recording Mode

Controlled by environment variable:

- `ROSEY_RECORD_FIXTURES=1`: Record new fixtures (requires live API keys)
- `ROSEY_RECORD_FIXTURES=0` (default): Use existing fixtures, fail if missing

### Fixture Format

```json
{
  "request": {
    "method": "GET",
    "url": "https://api.themoviedb.org/3/search/movie",
    "params": {"query": "The Matrix", "language": "en-US"}
  },
  "response": {
    "status": 200,
    "headers": {"Content-Type": "application/json"},
    "body": { /* actual API response */ }
  },
  "recorded_at": "2025-10-25T12:00:00Z",
  "expires_at": "2025-11-25T12:00:00Z"
}
```

### Fixture Matching

Match by:
1. Provider (TMDB/TVDB)
2. Endpoint
3. Query parameters (normalized, sorted)

### Fixture Management

- **Update fixtures**: Re-run tests with `ROSEY_RECORD_FIXTURES=1`
- **Prune old fixtures**: Delete unused files
- **Version fixtures**: Commit to git for reproducible CI

### Test Guidelines

```python
import pytest
from rosey.providers import TMDBProvider

@pytest.mark.asyncio
async def test_search_movie():
    # Uses recorded fixture by default
    provider = TMDBProvider(api_key="test_key", use_fixtures=True)
    results = await provider.search_movie("The Matrix")
    assert len(results) > 0
    assert results[0]["title"] == "The Matrix"

@pytest.mark.live  # Opt-in for live tests
@pytest.mark.skipif(not os.getenv("TMDB_API_KEY"), reason="No API key")
async def test_search_movie_live():
    # Hits live API (for fixture recording or validation)
    provider = TMDBProvider(api_key=os.getenv("TMDB_API_KEY"), use_fixtures=False)
    results = await provider.search_movie("The Matrix")
    assert len(results) > 0
```

## Caching Backend Decision

### Selected: DiskCache

**Rationale**:
- Simple, fast, and reliable
- No schema management
- Automatic eviction and size limits
- Good for small-to-medium caches (<1GB)

**Alternative Considered**: SQLite
- Pros: Structured queries, better for large datasets
- Cons: More complexity, schema migrations, overkill for simple KV cache

**Implementation**:
```python
from diskcache import Cache

cache = Cache(directory="~/.cache/rosey/metadata", size_limit=100 * 1024 * 1024)

def get_cached_metadata(key: str) -> dict | None:
    return cache.get(key)

def cache_metadata(key: str, value: dict, ttl_days: int = 30) -> None:
    cache.set(key, value, expire=ttl_days * 86400)
```
