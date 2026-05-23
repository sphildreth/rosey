# User Guide

Rosey organizes movie and TV files into Jellyfin-friendly folders.

This Rust rewrite is intended to support:

- scanning source folders
- identifying movies and TV episodes
- previewing destination paths
- dry-run mode
- safe move/copy operations
- sidecar file handling
- operation logs
- recovery after failed moves

## Planned workflow

1. Choose source folder.
2. Choose Movies target folder.
3. Choose TV target folder.
4. Scan.
5. Review confidence and destination plan.
6. Resolve conflicts.
7. Run dry-run.
8. Execute move/copy.
9. Review logs.

## Naming goals

Movies:

```text
Movies/Movie Title (Year)/Movie Title (Year).ext
```

TV:

```text
TV/Show Name/Season 01/Show Name - S01E02 - Episode Title.ext
```
