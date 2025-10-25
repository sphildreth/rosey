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

- Docs Index: [docs/README.md](./docs/README.md)
- Product Requirements: [docs/PRD.md](./docs/PRD.md)
- Technical Specification: [docs/TECH_SPEC.md](./docs/TECH_SPEC.md)
- UI Mockups: [docs/mockups/UI_MOCKUPS.md](./docs/mockups/UI_MOCKUPS.md)

## Status

This repository currently focuses on specifications and design. Implementation details and architecture decisions live in the Tech Spec. Binaries and build instructions will be added once development begins.

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
