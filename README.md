<p align="center">
	<img src="./graphics/logo.png" alt="Rosey logo" width="160" />
</p>

<h1 align="center">Rosey</h1>

<p align="center"><b>Media organizer for Jellyfin</b> — Scan, identify, and safely organize Movies and TV Shows into Jellyfin‑friendly folders.</p>

<p align="center">
	<a href="https://img.shields.io/badge/status-design%20phase-blue"><img alt="Project status" src="https://img.shields.io/badge/status-design%20phase-blue"></a>
	<a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-green"></a>
	<img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%26%20Linux-8A2BE2">
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

- Offline identification from filenames, folders, and .nfo files
- Optional online metadata: TMDB primary, TVDB optional (with caching and rate limiting)
- Confidence scoring with reasons (Green ≥70, Yellow 40–69, Red <40)
- Safe batch moves with rollback; atomic rename on same volume, copy‑verify across volumes
- Sidecar handling: .srt, .nfo, artwork moved with their media
- Responsive PySide6 UI with light/dark theme; tree + grid views; progress updates
- Jellyfin naming conventions for Movies and Shows
- Clear, rotating logs; dry‑run preview mode

## Documentation

- **User Guide**: [docs/SETUP.md](./docs/SETUP.md) — installation, configuration, troubleshooting
- Docs Index: [docs/README.md](./docs/README.md)
- Product Requirements: [docs/PRD.md](./docs/PRD.md)
- Technical Specification: [docs/TECH_SPEC.md](./docs/TECH_SPEC.md)
- UI Mockups: [docs/mockups/UI_MOCKUPS.md](./docs/mockups/UI_MOCKUPS.md)

## Status

Core features are implemented (scanning, identification, scoring, planning, moving, online providers, UI). Packaging and distribution are ready. See [docs/SETUP.md](./docs/SETUP.md) for user installation guide.

## Developer setup

These steps assume Linux or Windows with a Python 3.11 virtual environment.

Set up a virtual environment

```bash
# From repo root
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\\Scripts\\activate
python -m pip install --upgrade pip
```

Install dependencies

```bash
# When pyproject.toml is added (M1):
pip install -e ".[dev]"

# Until then, there may be no installable package; you can still install dev tools:
pip install pytest ruff mypy black
```

Run tests

```bash
pytest -q
```

Lint, type-check, and format

```bash
ruff check .
mypy .
black .
```

Run the application

```bash
python -m rosey.app
```

Build binary packages

```bash
# Linux/macOS
./scripts/build_package.sh

# Windows
scripts\build_package.bat
```

Run smoke tests on packaged binary

```bash
# Linux/macOS
./scripts/smoke_test.sh

# Windows
scripts\smoke_test.bat
```

Notes

- The project uses the "src layout": code lives under `src/`, the package is `src/rosey`, imported as `import rosey`.
- If UI tests need a display in CI, you can run them with `xvfb-run -a pytest -q` on Linux.
- For coding agents and exact prompts, see `docs/IMPLEMENTATION_GUIDE.md` and `docs/AI_AGENT_PLAYBOOK.md`.

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
