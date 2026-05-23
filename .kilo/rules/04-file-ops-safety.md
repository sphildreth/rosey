# File Operation Safety Rules

Rosey moves and copies user media files. Treat file operations as high-risk.

Do not implement destructive behavior unless all are true:

- There is a dry-run path.
- There are temp-dir integration tests.
- Errors are recoverable and clearly reported.
- Partial copies are cleaned up or journaled.
- Source files are not deleted until destination verification succeeds.
- Conflict behavior is explicit: skip, replace, or keep-both.
- Sidecar behavior is tested.
- Cross-volume behavior is considered separately from same-volume rename.

Never run tests against real user media paths. Use `tempfile` or fixtures.
