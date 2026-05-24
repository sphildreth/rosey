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
    "theme": "default",
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
    "confirm_delete": true,
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
    "movies_always_in_own_directory": true,
    "title_remove_segments": ["fan edit", "director commentary"]
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

The TUI reads configured source, movie target, TV target, theme, dry-run mode, confirm-delete mode, conflict policy, scan concurrency, symlink behavior, confidence thresholds, title remove segments, provider settings, and auto-delete patterns. On the Settings screen, use Up/Down to select a field, `e` to edit it, and `w` or `s` to save the runtime snapshot back to `rosey.json`. The Doctor screen reports whether configured paths, provider settings, cache paths, and system tools look usable.

The CLI reads configured source and target paths, scan concurrency, symlink behavior, conflict policy, confidence bands, identification settings, title remove segments, and provider settings. `rosey-cli doctor` reports configuration and system readiness, with `--json` for machine-readable output. `rosey-cli run --save-config` persists explicitly supplied path arguments. CLI live moves still require `--no-dry-run`; a stored `behavior.dry_run = false` does not make `rosey-cli run` destructive by default.

Rosey preserves the `ui` section when loading and saving config, although the terminal UI does not use desktop window geometry or splitter settings.

## Delete Confirmation

`behavior.confirm_delete` defaults to `true`. When enabled, the TUI asks before `Del` removes a selected item from the Move Plan or deletes a selected scan result's containing directory from disk. When disabled, those delete actions happen immediately.

## TUI Theme configuration

The TUI reads the `ui.theme` config key for its built-in palette.

- `default` (default): Rosey default terminal palette.
- `terminal`: Uses ANSI terminal colors with minimal custom RGB styling.
- `high_contrast`: High-contrast palette for low-visibility terminals.
- `no_color`: Disable colors for monochrome environments.
- `rainbow`: Bright multi-color palette.

Set it in `rosey.json`:

```json
"ui": {
  "theme": "rainbow"
}
```

If `ui.theme` contains an unknown value, the TUI falls back to `default` behavior and continues running.
Doctor reports the configured theme as unrecognized.

## Title Cleanup

Rosey removes common release tags and technical markers from parsed titles before matching, including resolution, codecs, sources, bit depth, audio markers, and bracketed release groups. `identification.title_remove_segments` adds your own case-insensitive title fragments to remove after normal cleanup.

```json
"identification": {
  "title_remove_segments": ["fan edit", "director commentary"]
}
```

During planning, movie identification first parses the filename. If the filename does not produce both a title and release year, Rosey tries the containing directory name. With online providers enabled, a movie that has both title and year will be searched against configured providers; Rosey auto-selects the provider identity when the search returns one result or a result exactly matches the parsed title and year.
