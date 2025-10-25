# Development Policies — Rosey

## Write-Scope Policy

### Allowed Modifications
- `src/rosey/**` — All application code
- `tests/**` — All test code
- `docs/**` — Documentation
- `pyproject.toml` — Project configuration
- `.pre-commit-config.yaml` — Pre-commit hooks
- `.github/workflows/**` — CI/CD configuration
- Package specs (e.g., PyInstaller `.spec` files when added)

### Prohibited Modifications
- `.git/**` — Git internals
- `venv/`, `.venv/`, `env/` — Virtual environments
- IDE-specific files (`.idea/`, `.vscode/` settings, etc.)
- Build artifacts (`dist/`, `build/`, `*.egg-info/`)

### Guidelines
- Keep changes surgical and minimal
- Prefer single-purpose commits
- Each change should be independently testable
- Update tests alongside code changes

---

## Diff Size Policy

### Preferred Sizes
- **Small**: <200 lines, <3 files — Ideal for most changes
- **Medium**: 200-500 lines, 3-8 files — Requires justification
- **Large**: >500 lines or >8 files — Requires supervisor approval

### Guidelines
- Break large changes into multiple small PRs
- Use feature flags for incremental rollouts
- Document why a change is larger than preferred
- Consider refactoring vs. new feature trade-offs

---

## Approval Gates

### Per-Commit Gates (Automated via CI)
1. **Lint**: `ruff check .` must pass
2. **Format**: `black --check .` must pass
3. **Type Check**: `mypy src/rosey` must pass with no errors
4. **Tests**: `pytest` must pass all tests
5. **Coverage**: No decrease in code coverage (enforced for critical paths)

### Per-Phase Gates (Human Review)
- **Phase Acceptance Criteria Met**: All checklist items verified
- **Documentation Updated**: Relevant docs reflect changes
- **Tests Added/Updated**: New functionality has test coverage
- **UI Smoke Test**: Application launches and basic flows work

### Merge Blockers
- Any CI check failure
- Unresolved review comments
- Merge conflicts
- Missing tests for new functionality
- Secrets or credentials committed

---

## Testing Policy

### Test Coverage Requirements
- **Core Logic**: ≥90% (scanner, identifier, scorer, planner, mover)
- **UI Code**: ≥60% (focus on model/signal tests, not widget rendering)
- **CLI**: ≥80%
- **Utilities**: ≥85%

### Test Types Required
1. **Unit Tests**: Pure functions, data transformations, business logic
2. **Integration Tests**: Module interactions, file I/O, API calls with fixtures
3. **Property Tests**: Hypothesis-based tests for parsers, path handlers
4. **UI Tests**: pytest-qt for signals/slots, model changes (not full rendering)
5. **Smoke Tests**: End-to-end flows in CI (headless mode)

### Test Guidelines
- Use fixtures for complex setup
- Mock external dependencies (filesystem, network)
- Recorded fixtures for provider API calls (default in CI)
- Live tests opt-in with explicit budget and rate limiting
- Property tests for edge cases and fuzz testing

---

## Secrets and Privacy Policy

### Secrets Handling
- **Never** commit secrets to version control
- Use `.env` files (gitignored) for local development
- Use OS keyring for persistent storage in production
- Redact secrets in all log output
- Use environment variables or config files for CI secrets

### Logging Redaction Rules
- API keys: Always redact
- Paths: Log relative paths where possible; full paths OK if no PII
- Filenames: OK to log; sanitize any user data in names
- Errors: Log error types/messages; redact any embedded secrets

### Privacy Principles
- No telemetry or analytics
- No external service calls without explicit user opt-in
- Cache provider responses locally to minimize API calls
- Clear documentation of all network activity

---

## Code Style Guidelines

### Python Style
- Follow PEP 8 (enforced by ruff)
- Use `black` formatting (line length: 100)
- Type hints required for public APIs (enforced by mypy)
- Docstrings for modules, classes, and public functions (Google style)

### Naming Conventions
- Classes: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `UPPER_SNAKE_CASE`
- Private members: `_leading_underscore`
- Type variables: `TitleCase` with descriptive names

### Import Order (enforced by ruff)
1. Standard library
2. Third-party packages
3. Local application imports

### Comments
- Focus on "why" not "what"
- Explain non-obvious decisions
- Link to issues/PRs for context
- Remove commented-out code (use git history)

---

## Concurrency and Threading

### UI Threading Rules
- **Never** block the UI thread
- Use `QThreadPool` + `QRunnable` for long-running tasks
- Communicate via Qt signals/slots
- Update UI elements only from the main thread

### File I/O Threading
- Scan operations: Background threads with progress signals
- Move operations: Background threads with atomic commits
- Config I/O: Main thread OK (small, infrequent)

### Thread Safety
- Use thread-safe data structures for shared state
- Protect critical sections with locks when necessary
- Prefer message passing over shared memory
- Document thread-safety guarantees in docstrings

---

## Error Handling

### Error Classification
- **Fatal**: Config corruption, missing dependencies → Exit with clear message
- **Recoverable**: Network errors, permission issues → Retry with backoff, log, continue
- **User Errors**: Invalid input, bad paths → Show error dialog, don't crash
- **Bugs**: Unexpected exceptions → Log stack trace, report to user, attempt recovery

### Error Reporting
- User-facing errors: Clear, actionable messages
- Log errors with context (operation, inputs, state)
- Include error codes for common issues (documented in troubleshooting)
- Never expose stack traces to end users (log only)

---

## Dependency Management

### Adding Dependencies
- Justify new dependencies in PR description
- Prefer established, maintained libraries
- Check license compatibility (MIT, Apache 2.0, BSD OK)
- Pin versions in `pyproject.toml`
- Update dependencies regularly, test thoroughly

### Dependency Guidelines
- Minimize dependency count
- Use Python standard library when feasible
- Consider vendoring small, stable utilities
- Document purpose of each dependency

---

## Documentation Standards

### Required Documentation
- **README.md**: Installation, quick start, basic usage
- **PRD.md**: Product requirements (stable reference)
- **TECH_SPEC.md**: Technical architecture
- **IMPLEMENTATION_GUIDE.md**: Agent-driven development plan
- **CHANGELOG.md**: User-facing changes per release

### Code Documentation
- Module docstrings: Purpose, key classes, usage examples
- Class docstrings: Purpose, attributes, usage
- Function docstrings: Parameters, returns, raises, examples
- Inline comments: Non-obvious decisions, workarounds, TODOs

### Documentation Updates
- Update docs in the same PR as code changes
- Keep examples up to date
- Document breaking changes prominently
- Add troubleshooting entries for new error modes

---

## Performance Guidelines

### Benchmarking
- Profile before optimizing
- Use `cProfile` or `py-spy` for profiling
- Document performance-critical sections
- Set performance budgets (e.g., scan 50k files in <30s)

### Optimization Priorities
1. Correctness and safety first
2. Responsiveness (UI, user-facing operations)
3. Throughput (batch operations, scans)
4. Memory efficiency (large libraries)

### Avoid Premature Optimization
- Simple, clear code first
- Optimize hot paths only
- Measure before and after
- Document trade-offs

---

## Release Process

### Version Numbering
- Semantic versioning: `MAJOR.MINOR.PATCH`
- Pre-releases: `0.x.y` (API not stable)
- Release candidates: `1.0.0-rc.1`

### Release Checklist
1. All tests pass on all platforms
2. Changelog updated
3. Version bumped in `pyproject.toml`
4. Git tag created: `v1.0.0`
5. Binaries built and tested (Windows, Linux)
6. Release notes published
7. Documentation updated on website/repo

---

## Backward Compatibility

### API Stability
- Public API stable after 1.0.0
- Deprecation warnings for breaking changes
- Migration guides for major versions
- Config file versioning and migration

### Config File Compatibility
- Validate config schema on load
- Migrate old formats automatically when possible
- Warn users of deprecated settings
- Preserve unknown fields for forward compatibility

---

## Contributor Workflow

### Development Setup
```bash
# Clone and enter repo
git clone https://github.com/yourorg/rosey.git
cd rosey

# Create virtual environment
python3.11 -m venv .venv
source .venv/bin/activate  # or .venv\Scripts\activate on Windows

# Install in editable mode with dev dependencies
pip install -e ".[dev]"

# Install pre-commit hooks
pre-commit install

# Run tests
pytest -q

# Run app
python -m rosey.app
```

### Making Changes
1. Create a branch: `git checkout -b feature/my-feature`
2. Make changes following policies above
3. Add tests for new functionality
4. Run quality checks: `pytest && ruff check . && black . && mypy src/rosey`
5. Commit with clear message
6. Push and create PR
7. Address review feedback
8. Merge after approval and passing CI

---

## Contact and Questions

- Issues: GitHub Issues
- Discussions: GitHub Discussions
- Security: See SECURITY.md (if/when added)
