# Configuration

Rosey reads `rosey.json` from the platform config directory:

- Linux/macOS: `$XDG_CONFIG_HOME/rosey/rosey.json`, or `~/.config/rosey/rosey.json`
- Windows: `%APPDATA%\rosey\rosey.json`

If the file is missing or invalid, Rosey uses built-in defaults.

## Example

```json
{
  "version": "1.0",
  "paths": {
    "source": "/mnt/incoming",
    "movies": "/mnt/media/Movies",
    "tv": "/mnt/media/TV"
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
    "conflict_policy": "skip",
    "auto_delete_patterns": ["sample.mkv", "*.nfo", "*.txt"]
  },
  "scanning": {
    "concurrency_local": 8,
    "concurrency_network": 2,
    "follow_symlinks": false,
    "enforce_one_media_per_folder": false
  },
  "identification": {
    "use_online_providers": false,
    "confidence_thresholds": {
      "green": 70,
      "yellow": 40
    },
    "prefer_nfo_ids": true,
    "minimum_movie_duration_minutes": 60,
    "movies_always_in_own_directory": true
  },
  "providers": {
    "tmdb_api_key": "",
    "tmdb_language": "en-US",
    "tmdb_region": "US",
    "tvdb_api_key": "",
    "tvdb_language": "eng",
    "cache_ttl_days": 30
  },
  "logging": {
    "level": "INFO",
    "file_path": "",
    "max_file_size_mb": 10,
    "backup_count": 5,
    "redact_secrets": true,
    "log_to_console": false
  }
}
```

## Current Support

The TUI reads configured source, movie target, TV target, dry-run mode, conflict policy, scan concurrency, symlink behavior, confidence thresholds, provider settings, and auto-delete patterns. On the Settings screen, use Up/Down to select a field, `e` to edit it, and `w` or `s` to save the runtime snapshot back to `rosey.json`. The Doctor screen reports whether configured paths, provider settings, cache paths, and system tools look usable.

The CLI reads configured source and target paths, scan concurrency, symlink behavior, conflict policy, confidence bands, identification settings, and provider settings. `rosey-cli doctor` reports configuration and system readiness, with `--json` for machine-readable output. `rosey-cli run --save-config` persists explicitly supplied path arguments. CLI live moves still require `--no-dry-run`; a stored `behavior.dry_run = false` does not make `rosey-cli run` destructive by default.

Rosey preserves the `ui` section when loading and saving config, although the terminal UI does not use desktop window geometry or splitter settings.
