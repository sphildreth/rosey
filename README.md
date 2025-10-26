<p align="center">
	<img src="./graphics/logo.png" alt="Rosey logo" width="160" />
</p>

<h1 align="center">Rosey</h1>

<p align="center"><b>Media organizer for Jellyfin</b> — Scan, identify, and safely organize Movies and TV Shows into Jellyfin‑friendly folders.</p>

<p align="center">
	<a href="https://img.shields.io/badge/status-beta-brightgreen"><img alt="Project status" src="https://img.shields.io/badge/status-beta-brightgreen"></a>
	<a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-green"></a>
	<img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%26%20Linux-8A2BE2">
	<img alt="Python" src="https://img.shields.io/badge/python-3.13-blue">
	<img alt="GUI" src="https://img.shields.io/badge/GUI-PySide6-41b883">
</p>

<p align="center">
	<a href="#features">Features</a> •
	<a href="#documentation">Documentation</a> •
	<a href="#developer-setup">Developer setup</a> •
	<a href="#contributing">Contributing</a> •
	<a href="#license">License</a>
</p>

---

Rosey is a cross‑platform desktop utility (Windows + Linux) that scans a Source folder (including network shares), identifies Movies and TV Shows using offline signals and optional online metadata lookups (TMDB/TVDB), then moves selected items into clean, Jellyfin‑ready folders. Privacy‑first: no telemetry; provider calls only when enabled.

## Features

- **Offline identification** from filenames, folders, and .nfo files
- **Optional online metadata**: TMDB primary, TVDB optional (with caching and rate limiting)
- **Confidence scoring** with reasons (Green ≥70, Yellow 40–69, Red <40)
- **Safe batch moves** with rollback; atomic rename on same volume, copy‑verify across volumes
- **Sidecar handling**: .srt, .nfo, artwork moved with their media
- **Auto-delete patterns**: Configurable cleanup of sample files and unwanted content after successful moves
- **Empty directory cleanup**: Automatically removes empty source directories after moving content
- **Conflict resolution**: Interactive dialog for handling file conflicts with skip/replace/keep both options
- **Episode title extraction**: Captures episode titles from filenames when available
- **Responsive PySide6 UI** with light/dark theme; tree + grid views; progress updates
- **Jellyfin naming conventions** for Movies and TV Shows
- **Clear, rotating logs**; dry‑run preview mode

## Documentation

- **User Guide**: [docs/SETUP.md](./docs/SETUP.md) — installation, configuration, troubleshooting
- **Docs Index**: [docs/README.md](./docs/README.md) — complete documentation overview
- **Product Requirements**: [docs/PRD.md](./docs/PRD.md) — goals, scope, and user stories
- **Technical Specification**: [docs/TECH_SPEC.md](./docs/TECH_SPEC.md) — architecture and design
- **AI Agent Guide**: [docs/AI_AGENT_PLAYBOOK.md](./docs/AI_AGENT_PLAYBOOK.md) — development workflow
- **Configuration Schema**: [docs/CONFIG_SCHEMA.md](./docs/CONFIG_SCHEMA.md) — settings reference

## Status

**Beta** — All core features are implemented and tested:
- ✅ Scanning (local and network paths)
- ✅ Identification (offline + online with TMDB/TVDB)
- ✅ Scoring and confidence display
- ✅ Planning and conflict resolution
- ✅ Safe file operations with rollback
- ✅ Comprehensive UI with progress tracking
- ✅ Packaging and distribution (Windows/Linux)
- ✅ 650+ tests passing

See [docs/SETUP.md](./docs/SETUP.md) for installation instructions.

## Developer setup

**Requirements**: Python 3.13+

### Set up a virtual environment

```bash
# From repo root
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
python -m pip install --upgrade pip
```

### Install dependencies

```bash
# Install package with dev dependencies
pip install -e ".[dev]"
```

### Run tests

```bash
pytest
```

### Run quality checks

```bash
# All checks (lint, format, type-check, test)
./scripts/test_quality.sh  # Linux/macOS
scripts\test_quality.bat    # Windows

# Individual checks
ruff check .                # Lint
ruff format --check .       # Format check
mypy .                      # Type check
```

### Run the application

```bash
python -m rosey.app
```

### Build binary packages

```bash
# Linux/macOS
./scripts/build_package.sh

# Windows
scripts\build_package.bat
```

### Run smoke tests on packaged binary

```bash
# Linux/macOS
./scripts/smoke_test.sh

# Windows
scripts\smoke_test.bat
```

### Notes

- The project uses the **"src layout"**: code lives under `src/rosey/`, imported as `import rosey`
- **18 test modules** with 650+ test cases covering identification, configuration, file operations, and UI
- UI tests can run headless on Linux with: `xvfb-run pytest`
- For AI coding agents, see [docs/AI_AGENT_PLAYBOOK.md](./docs/AI_AGENT_PLAYBOOK.md)

## Contributing

**Using AI coding agents?** Start with **docs/AI_AGENT_PLAYBOOK.md**.
It constrains scope, requires unified diffs, and defines pass/fail per step so patches actually run.

Contributions are welcome! A few tips to get started:

- Read the [PRD](./docs/PRD.md) to understand goals and scope
- Review the [Technical Spec](./docs/TECH_SPEC.md) for design and component boundaries
- Discuss larger changes in an issue before opening a PR
- Keep docs updated when behavior or decisions change

## License

MIT — see [LICENSE](./LICENSE).
