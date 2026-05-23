# Parity Strategy

## Goal

Preserve the good behavior from Python Rosey without dragging Python architecture into the Rust rewrite.

## Golden-master approach

Use Python Rosey to generate expected JSON outputs for known inputs.

Rust should produce matching JSON for equivalent commands.

Examples:

```bash
# Python repo
cd ../rosey
python -m rosey.devtools.golden scan tests/fixtures/tree_001 > ../rosey-rust/tests/golden/scan_001.json

# Rust repo
cd ../rosey-rust
cargo run -p rosey-cli -- scan tests/fixtures/tree_001 --json > target/scan_001.actual.json
```

Then compare normalized JSON.

## What deserves golden files

- filename parsing
- episode extraction
- year/date extraction
- NFO parsing
- Jellyfin destination planning
- conflict suffix generation
- scan results on fixture trees
- dry-run move plans
- sidecar discovery
- failure/preflight behavior

## What should not be golden-matched exactly

- PySide6 UI behavior
- Python exception text
- log line formatting
- internal implementation details
- provider responses that change over time

## Normalization rules

Golden outputs should avoid unstable data:

- absolute temp paths
- timestamps
- OS-specific separators where avoidable
- unordered maps
- transient provider fields
- durations/timing

Prefer normalized paths and deterministic ordering.

## Rust test tools

- `insta` for snapshots
- `serde_json` for normalized JSON
- `pretty_assertions` for readable diffs
- `tempfile` for filesystem tests
- `proptest` for parser/sanitizer properties

## Done definition

A migrated module is done when:

- Rust unit tests cover the normal cases
- Rust integration tests cover relevant filesystem cases
- golden parity exists for high-value behavior
- docs or ADRs note intentional deviations from Python behavior
