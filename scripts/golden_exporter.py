#!/usr/bin/env python3
"""Golden output exporter for Rosey parity testing.

Run from the root of the Rosey repo:

    python scripts/golden_exporter.py

Generates golden JSON files in tests/golden/ using the Python Rosey reference
implementation. Requires the Python Rosey repo at ``../rosey`` to be importable.

The Python Rosey repo MUST be installed or PYTHONPATH set appropriately.
"""

import json
import sys
import os
from pathlib import Path

def ensure_python_rosey_importable():
    py_rosey = Path(__file__).resolve().parent.parent.parent.parent / "rosey"
    if str(py_rosey / "src") not in sys.path:
        sys.path.insert(0, str(py_rosey / "src"))

GOLDEN_DIR = Path(__file__).resolve().parent.parent / "tests" / "golden"
FIXTURES_DIR = Path(__file__).resolve().parent.parent / "tests" / "fixtures"

def serialize(obj):
    """Serialize objects to JSON with normalized form."""
    if hasattr(obj, 'to_dict'):
        return serialize(obj.to_dict())
    if hasattr(obj, '__dict__'):
        return serialize(vars(obj))
    if isinstance(obj, Path):
        # Normalize paths to use forward slashes
        return str(obj).replace('\\\\', '/')
    if hasattr(obj, '__dataclass_fields__'):
        import dataclasses
        return serialize(dataclasses.asdict(obj))
    return obj

class GoldenEncoder(json.JSONEncoder):
    def default(self, obj):
        return serialize(obj) or super().default(obj)

def dump_golden(name, data):
    path = GOLDEN_DIR / f"{name}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, 'w') as f:
        json.dump(data, f, cls=GoldenEncoder, indent=2, sort_keys=True)
    print(f"  Wrote {path}")

def main():
    ensure_python_rosey_importable()

    try:
        from rosey.scanner import scan_directory
        from rosey.identifier import identify_file
        from rosey.planner import plan_path
        from rosey.identifier.patterns import (
            extract_year, extract_episode_info, extract_part, extract_date,
            clean_title, extract_season_from_folder
        )
        from rosey.identifier.nfo import parse_nfo
        from rosey.mover import discover_sidecars
    except ImportError as e:
        print(f"ERROR: Cannot import Python Rosey. Set PYTHONPATH or install rosey: {e}")
        sys.exit(1)

    print("Generating golden outputs from Python Rosey...")

    # ── Scanner ───────────────────────────────────────────────────────────

    media_tree = FIXTURES_DIR / "media_tree_001"
    if media_tree.exists():
        print("\\n[scanner]")
        results = scan_directory(str(media_tree), max_workers=8)
        dump_golden("scan_media_tree_001", [serialize(r) for r in results])

    # ── Parser ───────────────────────────────────────────────────────────

    print("\\n[parser]")
    test_filenames = [
        "The.Matrix.1999.1080p.BluRay.x264.mkv",
        "Inception.2010.1080p.BluRay.x264-GROUP.mkv",
        "Example Show S01E01 Pilot.mkv",
        "Example.Show.S01E02.mkv",
        "Example Show 1x03.mkv",
        "Daily Show 2015-03-15.mkv",
        "Movie.Part.2.2005.mkv",
        "Movie Part II.mkv",
        "S02E03 Some Episode Title.mkv",
        "Some.Show.Season.01.EP04.mkv",
        "Some Show S01E01-E05.mkv",
        "12 Monkeys (1995).mkv",
        "Movie.Vol.2.2010.mkv",
        "Show.Name.S01E01.1080p.WEB-DL.x264.mkv",
        "Mad Max Fury Road 2015 1080p.mkv",
        "The.Wire.S03E01.1080p.HEVC.x265.mkv",
    ]

    parser_results = {}
    for fname in test_filenames:
        year = extract_year(fname)
        ep_info = extract_episode_info(fname, known_season=None)
        part = extract_part(fname)
        date = extract_date(fname)
        title = clean_title(fname)

        parser_results[fname] = {
            "year": year,
            "episode_info": serialize(ep_info) if ep_info else None,
            "part": part,
            "date": serialize(date) if date else None,
            "title": title,
        }

    dump_golden("parser_filenames", parser_results)

    # ── Season folder ─────────────────────────────────────────────────────

    print("\\n[season_folder]")
    season_tests = {
        "Season 01": extract_season_from_folder("Season 01"),
        "Season 1": extract_season_from_folder("Season 1"),
        "S03": extract_season_from_folder("S03"),
        "Random Folder": extract_season_from_folder("Random Folder"),
        "season 05": extract_season_from_folder("season 05"),
    }
    dump_golden("season_folder", season_tests)

    # ── NFO ───────────────────────────────────────────────────────────────

    print("\\n[nfo]")
    nfo_dir = FIXTURES_DIR / "nfo"
    nfo_results = {}
    movie_nfo = nfo_dir / "movie.nfo"
    if movie_nfo.exists():
        result = parse_nfo(str(movie_nfo))
        nfo_results["movie.nfo"] = serialize(result) if result else None

    tvshow_nfo = nfo_dir / "tvshow.nfo"
    if tvshow_nfo.exists():
        result = parse_nfo(str(tvshow_nfo))
        nfo_results["tvshow.nfo"] = serialize(result) if result else None

    dump_golden("nfo", nfo_results)

    # ── Planner ───────────────────────────────────────────────────────────

    print("\\n[planner]")
    from rosey.models import MediaItem, MediaKind

    plan_tests = {}
    test_items = [
        {
            "name": "movie_basic",
            "item": MediaItem(
                kind=MediaKind.Movie,
                source_path="/source/The Matrix (1999).mkv",
                title="The Matrix",
                year=1999,
            )
        },
        {
            "name": "movie_with_tmdb",
            "item": MediaItem(
                kind=MediaKind.Movie,
                source_path="/source/Pulp Fiction.mkv",
                title="Pulp Fiction",
                year=1994,
                nfo={"tmdbid": "680"},
            )
        },
        {
            "name": "episode_basic",
            "item": MediaItem(
                kind=MediaKind.Episode,
                source_path="/source/tv/Show/S01/Show - S01E02.mkv",
                title="Show Name",
                season=1,
                episodes=[2],
            )
        },
        {
            "name": "episode_multi",
            "item": MediaItem(
                kind=MediaKind.Episode,
                source_path="/source/tv/Show/S01/Show - S01E01-E05.mkv",
                title="Show Name",
                season=1,
                episodes=[1, 2, 3, 4, 5],
            )
        },
        {
            "name": "episode_date",
            "item": MediaItem(
                kind=MediaKind.Episode,
                source_path="/source/tv/Daily/2015-03-15.mkv",
                title="Daily Show",
                date="2015-03-15",
            )
        },
    ]

    for test in test_items:
        dest = plan_path(test["item"], movies_root="/movies", tv_root="/tv")
        plan_tests[test["name"]] = str(dest)

    dump_golden("planner", plan_tests)

    # ── Sidecars ──────────────────────────────────────────────────────────

    print("\\n[sidecars]")
    sidecar_dir = FIXTURES_DIR / "media_tree_sidecars" / "movie"
    sidecar_results = {}
    avatar_path = sidecar_dir / "Avatar (2009).mkv"
    if avatar_path.exists():
        sidecars = discover_sidecars(str(avatar_path))
        sidecar_results["avatar"] = [str(s) for s in sidecars]
    dump_golden("sidecars", sidecar_results)

    print("\\nDone.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
