# Rosey Rust Technical Specification

Status: Draft

## 1. Architecture

Rosey Rust is a workspace of focused crates:

```text
rosey-core        domain models, parser helpers, naming/planning primitives
rosey-fs          scanner, sidecar discovery, transfer engine
rosey-metadata    metadata provider interfaces, cache boundary
rosey-cli         command-line interface and parity harness
rosey-tui         Ratatui/Crossterm terminal UI
```

Dependency direction:

```text
rosey-core
  ↑
rosey-fs
  ↑
rosey-cli
  ↑
rosey-tui

rosey-metadata depends on rosey-core only.
```

The core engine must not depend on the TUI.

## 2. Domain models

Core models are defined in `rosey-core`.

Important concepts:

- `MediaKind`
- `MediaItem`
- `IdentificationResult`
- `Score`
- `ConfidenceBand`
- `ConflictPolicy`
- `MovePlan`
- `MoveResult`

Models should serialize to JSON so they can be used by:

- CLI output
- golden-master tests
- TUI event/state snapshots
- debugging logs

## 3. Scanner

Scanner responsibilities:

- recursively enumerate source roots
- detect video files
- skip non-video files
- avoid crashing on permission errors
- preserve enough error detail for logs and UI
- support future local/network concurrency tuning

Initial implementation may be synchronous and simple. Later implementations should stream events to the TUI.

Planned scan events:

```rust
enum ScanEvent {
    Started { root: Utf8PathBuf },
    FileDiscovered { path: Utf8PathBuf, size_bytes: u64 },
    FileSkipped { path: Utf8PathBuf, reason: String },
    Error { path: Utf8PathBuf, error: String },
    Completed { total_files: u64, video_files: u64 },
}
```

## 4. Identifier

Identifier responsibilities:

- parse filename patterns
- extract years
- extract episodes and ranges
- parse daily-show dates
- parse parts/multipart indicators
- inspect folder context
- parse NFO files
- optionally call metadata providers

Port this from Python in stages:

1. filename patterns
2. folder context
3. NFO parsing
4. sidecar/companion discovery
5. duration probing
6. online metadata

## 5. Planner

Planner responsibilities:

- generate Jellyfin-friendly destination paths
- sanitize invalid characters
- preserve extensions
- handle multi-episode and multipart files
- detect conflicts
- apply conflict policies

Naming examples:

```text
Movies/The Matrix (1999)/The Matrix (1999).mkv
TV/Example Show/Season 01/Example Show - S01E02 - Episode Title.mkv
```

## 6. Mover

Mover responsibilities:

- dry-run by default
- preflight checks
- same-volume rename fast path
- cross-volume copy + verify + delete
- progress events
- sidecar handling
- operation journal
- rollback/recovery

Starter skeleton intentionally does not implement destructive moves.

## 7. Operation journal

Use JSON Lines:

```json
{"op":"copy_started","src":"...","dst":"...","bytes":123}
{"op":"copy_verified","src":"...","dst":"..."}
{"op":"source_deleted","src":"..."}
{"op":"completed","src":"...","dst":"..."}
```

Journal goals:

- crash visibility
- resume/repair support
- easier bug reports
- safer destructive file operations

## 8. CLI

The CLI is both a user tool and a migration harness.

Planned commands:

```bash
rosey scan <source> --json
rosey identify <path> --json
rosey plan --input candidates.json --movies <dir> --tv <dir> --json
rosey move --plan plan.json --dry-run
rosey move --plan plan.json --execute
```

## 9. TUI

TUI screens:

- Dashboard
- Scan Results
- Plan Preview
- Transfer Queue
- Logs / Recovery
- Settings
- Help

Keyboard-first commands:

```text
s     scan
p     preview plan
d     dry-run
m     move/copy execute
c     conflict policy
/     filter/search
tab   next panel
?     help
q     quit
```

## 10. Testing

Use:

- unit tests for parser/planner
- integration tests for filesystem behavior
- golden JSON parity tests
- `insta` for snapshots
- `proptest` for parser/sanitizer properties
- temp directories for mover tests

## 11. Logging

Use `tracing`.

Log:

- source and destination paths
- bytes transferred
- duration
- retries
- errors
- journal path
- recovery recommendation

Never log API keys.
