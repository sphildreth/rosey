# Cutover Checklist

Use this when replacing the Python Rosey repo with the Rust rewrite.

## Before cutover

- [ ] Rust CLI supports scan/identify/plan/move dry-run
- [ ] TUI supports scan/results/preview/dry-run
- [ ] mover has operation journal
- [ ] mover has temp-dir integration tests
- [ ] key Python golden parity tests pass
- [ ] README describes Rust version accurately
- [ ] user docs are updated
- [ ] release artifacts are tested on Linux
- [ ] release artifacts are tested on Windows

## Repository changes

- [ ] tag final Python release, e.g. `v0.1-python-final`
- [ ] rename old `rosey` repo to `rosey-python-archive`
- [ ] rename `rosey-rust` repo to `rosey`
- [ ] update GitHub repository description
- [ ] update badges
- [ ] update links in docs
- [ ] update package/release names
- [ ] create first Rust release

## After cutover

- [ ] keep old Python archive read-only
- [ ] document migration notes
- [ ] close obsolete Python-only issues
- [ ] migrate relevant issues manually if needed
