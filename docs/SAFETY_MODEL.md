# Safety Model

Rosey handles valuable media libraries, so destructive operations must be explicit and recoverable.

## Principles

- Dry-run by default.
- Preview before execution.
- Never delete source files until cross-volume copies are verified.
- Same-volume moves should use atomic rename/replace when possible.
- Cross-volume moves should copy, verify, then delete source.
- Multi-file media moves should be treated as one operation.
- Operation journal should be written before and during destructive steps.
- Failed operations should leave a clear recovery path.

## Conflict policies

- `skip`: leave existing destination untouched.
- `replace`: replace destination only when explicitly selected.
- `keep_both`: generate suffixes such as `Movie (1).mkv`.

## Recovery

The planned journal file is JSON Lines so interrupted operations can be inspected and resumed or rolled back where safe.
