# Rosey Rust PRD

Status: Draft  
Audience: product/design/implementation agents  
Repo: `sphildreth/rosey-rust`

## 1. Product summary

Rosey Rust is a high-performance rewrite of Rosey, a FOSS media organizer aimed at arranging shows and movies into a format Jellyfin likes.

The Rust version should preserve the useful behavior from the Python/PySide6 version while improving:

- packaging simplicity
- scan and transfer performance
- reliability on local and network filesystems
- maintainability
- coding-agent friendliness
- terminal-first power-user workflow

## 2. Target users

Primary users:

- Jellyfin users
- self-hosters
- NAS users
- power users who organize downloaded media
- users comfortable with terminals, SSH, SMB/NFS paths, and batch tools

Secondary users:

- casual desktop users who may later benefit from a GUI frontend

## 3. Goals

- Rust core engine for scanning, identifying, planning, and moving media.
- CLI for automation and parity testing.
- Ratatui/Crossterm TUI for the main interactive experience.
- Safe file operations with dry-run, preflight, rollback, and journaling.
- Golden-master parity with existing Python behavior for important cases.
- No web server requirement.
- No Python virtual environment.
- Easy release binaries.

## 4. Non-goals

- No web UI in the initial Rust rewrite.
- No desktop GUI in the initial Rust rewrite.
- No destructive move behavior without dry-run and tests.
- No online metadata dependency for basic organization.
- No telemetry.

## 5. Core user stories

### Scan

As a user, I want Rosey to scan a source folder recursively so I can see which media files it found.

Acceptance criteria:

- detects supported video extensions
- skips non-video files
- reports errors without crashing
- supports local and network paths
- can emit JSON via CLI

### Identify

As a user, I want Rosey to infer whether a file is a movie or TV episode from filenames, folders, NFO files, and optional metadata.

Acceptance criteria:

- recognizes common episode patterns
- recognizes movie year patterns
- handles daily-show dates
- supports NFO-driven identification
- explains confidence reasons

### Plan

As a user, I want to preview Jellyfin destination paths before moving files.

Acceptance criteria:

- supports Movies and TV naming conventions
- preserves file extension
- handles multi-episode files
- sanitizes invalid path characters
- reports conflicts

### Move/copy

As a user, I want Rosey to move files safely and recoverably.

Acceptance criteria:

- dry-run is default
- same-volume moves use rename/replace semantics where safe
- cross-volume moves copy, verify, then delete source
- sidecars move with media
- conflict policies are explicit
- operation journal records destructive steps

### TUI

As a user, I want a keyboard-first terminal UI that shows scan results, plans, progress, and logs.

Acceptance criteria:

- responsive during long operations
- clear status/progress display
- help screen for shortcuts
- supports preview and confirmation before destructive operations

## 6. Success metrics

- Rust CLI can match Python golden outputs for the key scanner/parser/planner/mover behaviors.
- Large scans remain responsive in TUI.
- Move/copy operations produce clear journals and logs.
- Users can download a binary and run it without Python or a venv.
- Agent-driven development can continue one module at a time with low regression risk.
