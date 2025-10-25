# Rosey

**Media organizer for Jellyfin** — A cross-platform desktop utility to scan, identify, and organize your movie and TV show files into Jellyfin-friendly folders.

## Features

- **Offline & Online Identification**: Detects media from filenames, folders, and .nfo files; optional TMDB/TVDB lookups for better accuracy.
- **Confidence Scoring**: Green/Yellow/Red ratings with tooltips explaining matches.
- **Safe Batch Moves**: Atomic renames or cross-volume copies with rollback on errors; handles sidecar files (.srt, .nfo, etc.).
- **Responsive UI**: PySide6-based interface with light/dark themes, tree/grid views, and progress tracking.
- **Privacy-First**: No telemetry; API keys stored securely; works offline.

## Installation

### Prerequisites
- Python 3.11+
- PySide6, httpx, pydantic, keyring, and other dependencies (see `requirements.txt`).

### From Source
1. Clone the repo: `git clone https://github.com/sphildreth/rosey.git`
2. Install dependencies: `pip install -r requirements.txt`
3. Run: `python src/app.py`

### Binaries (Coming Soon)
Download from releases for Windows/Linux.

## Usage

1. Set source folder, movie/TV targets in config.
2. Click **Scan** to identify files.
3. Review and select items (auto-select Green).
4. Click **Move Selected** to organize.

See [PRD](./docs/PRD.md) for details and [TECH_SPEC](./docs/TECH_SPEC.md) for architecture.

## Contributing

- Follow the [TECH_SPEC](./docs/TECH_SPEC.md) for code structure.
- Run tests: `pytest`
- Report issues or PRs welcome.

## License

See [LICENSE](./LICENSE).
