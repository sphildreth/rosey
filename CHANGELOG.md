# Changelog

All notable changes to Rosey will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Phase A — Planning & Guardrails)
- Initial project structure: `src/rosey/`, `tests/`, `docs/`
- Core configuration system with `RoseyConfig` Pydantic models
- Config file support (`rosey.json`) with platform-specific paths
- Data models: `MediaItem`, `IdentificationResult`, `Score`, `MovePlan`, `MoveResult`
- Development tooling: pytest, ruff, black, mypy, hypothesis, pytest-qt
- Pre-commit hooks for automated code quality checks
- CI/CD pipeline with lint, test, and UI smoke test jobs
- Comprehensive documentation:
  - `DEVELOPMENT_POLICIES.md` — Write-scope, diff size, approval gates
  - `CONFIG_SCHEMA.md` — Configuration schema and recording strategy
  - `tests/fixtures/README.md` — Fixture recording and management guide
- Test suite with 13 passing tests for config and models
- Virtual environment setup with all dev dependencies

### Infrastructure
- GitHub Actions CI workflow for Linux and Windows
- Multi-version Python testing (3.11, 3.12)
- Code coverage reporting via pytest-cov
- Type checking with mypy (strict mode)
- Code formatting with black (line length: 100)
- Linting with ruff (pycodestyle, pyflakes, isort, pyupgrade, etc.)

### Configuration
- Dry-run mode enabled by default for safety
- Online providers disabled by default (privacy-first)
- Configurable concurrency for local vs. network scans
- Theme support (system, light, dark)
- Logging with redaction for secrets
- Caching backend decision: DiskCache for metadata

### Quality Gates
- All tests pass (13/13)
- Ruff lint checks pass
- Black formatting checks pass
- Mypy type checks pass (strict mode)
- App launches without errors

## [0.1.0] - 2025-10-25 (Phase A Complete)

### Phase A Acceptance Criteria Met
- [x] Repository structure established: `src/rosey/**`, `tests/**`, `docs/**`
- [x] Pre-commit hooks configured: ruff, black, mypy
- [x] Pytest configuration with hypothesis integration
- [x] CI configured with lint, typecheck, tests, UI smoke test lanes
- [x] Config schema defined (`rosey.json`) with Pydantic models
- [x] Logging and redaction policy documented
- [x] Caching backend selected: DiskCache
- [x] Recording strategy for provider fixtures documented
- [x] Development policies documented: write-scope, diff size, approval gates
- [x] Basic app entry point with config loading
- [x] Core data models with full test coverage

### Development Workflow Established
- Virtual environment with PySide6 6.10.0
- Test command: `pytest -q`
- Lint command: `ruff check .`
- Format command: `black .`
- Type check command: `mypy src/rosey`
- Run app: `python -m rosey.app`

---

## Release Notes Format (for future releases)

### Added
For new features.

### Changed
For changes in existing functionality.

### Deprecated
For soon-to-be removed features.

### Removed
For now removed features.

### Fixed
For any bug fixes.

### Security
In case of vulnerabilities.
