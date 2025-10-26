"""Integration test demonstrating scan enhancements workflow."""

from pathlib import Path
from typing import Any, cast

import pytest

from rosey.grouper import MediaGroup, build_media_groups
from rosey.identifier import identify_file
from rosey.models import MediaItem, Score
from rosey.planner import plan_path
from rosey.scanner import scan_directory
from rosey.scorer import score_identification


@pytest.fixture
def complete_media_structure(tmp_path):
    """Create a complete media structure with movies and shows."""
    # Movie 1
    movie1_dir = tmp_path / "The Matrix (1999)"
    movie1_dir.mkdir()
    (movie1_dir / "The Matrix (1999).mkv").write_text("video")
    (movie1_dir / "The Matrix (1999).srt").write_text("subtitle")
    (movie1_dir / "movie.nfo").write_text(
        """<?xml version="1.0"?>
<movie>
    <title>The Matrix</title>
    <year>1999</year>
    <tmdbid>603</tmdbid>
</movie>"""
    )

    # Movie 2
    movie2_dir = tmp_path / "Inception (2010)"
    movie2_dir.mkdir()
    (movie2_dir / "Inception (2010).mkv").write_text("video")

    # TV Show
    show_dir = tmp_path / "Breaking Bad"
    season1 = show_dir / "Season 01"
    season2 = show_dir / "Season 02"
    season1.mkdir(parents=True)
    season2.mkdir(parents=True)

    (season1 / "Breaking Bad - S01E01.mkv").write_text("video")
    (season1 / "Breaking Bad - S01E01.srt").write_text("subtitle")
    (season1 / "Breaking Bad - S01E02.mkv").write_text("video")
    (season2 / "Breaking Bad - S02E01.mkv").write_text("video")

    (show_dir / "tvshow.nfo").write_text(
        """<?xml version="1.0"?>
<tvshow>
    <title>Breaking Bad</title>
    <year>2008</year>
    <tmdbid>1396</tmdbid>
</tvshow>"""
    )

    return tmp_path


def test_complete_workflow(complete_media_structure, tmp_path):
    """Test complete scan-to-plan workflow with grouping."""
    source_path = str(complete_media_structure)
    movies_root = str(tmp_path / "Movies")
    tv_root = str(tmp_path / "TV Shows")

    # Step 1: Scan
    scan_results = scan_directory(source_path)
    video_files = [r.path for r in scan_results if r.is_video]
    assert len(video_files) == 5  # 2 movies + 3 TV episodes (one was not created)

    # Step 2: Build media groups
    groups: list[MediaGroup] = build_media_groups(video_files, source_path)

    # Should have 3 groups: 2 movies, 1 show
    assert len(groups) == 3

    # Verify group classifications
    movie_groups = [g for g in groups if g.kind == "movie"]
    show_groups = [g for g in groups if g.kind == "show"]
    assert len(movie_groups) == 2
    assert len(show_groups) == 1

    # Step 3: Process each group
    all_items: list[dict[str, Any]] = []
    for group in groups:
        for video_path in group.primary_videos:
            # Identify
            ident_result = identify_file(video_path)

            # Score
            score = score_identification(ident_result)

            # Plan
            destination = plan_path(ident_result.item, movies_root=movies_root, tv_root=tv_root)

            all_items.append(
                {
                    "item": ident_result.item,
                    "score": score,
                    "destination": destination,
                    "group": group,
                }
            )

    # Verify we have all items
    assert len(all_items) == 5

    # Verify movie items
    movie_items = []
    for i in all_items:
        item = cast(MediaItem, i["item"])
        if item.kind == "movie":
            movie_items.append(i)
    assert len(movie_items) == 2
    for item_dict in movie_items:
        dest = cast(str, item_dict["destination"])
        score = cast(Score, item_dict["score"])
        assert movies_root in dest
        assert score.confidence > 0

    # Verify TV items
    tv_items = []
    for i in all_items:
        item = cast(MediaItem, i["item"])
        if item.kind == "episode":
            tv_items.append(i)
    assert len(tv_items) == 3
    for item_dict in tv_items:
        dest = cast(str, item_dict["destination"])
        score = cast(Score, item_dict["score"])
        assert tv_root in dest
        assert "Breaking Bad" in dest
        assert score.confidence > 0

    # Verify companions were discovered
    for group in groups:
        if group.kind == "movie" and "Matrix" in group.directory:
            # Matrix has a companion subtitle
            assert len(group.companions) > 0
        elif group.kind == "show":
            # Breaking Bad has companion subtitles
            assert len(group.companions) > 0


def test_media_group_ui_tree_representation(complete_media_structure):
    """Test how groups would be represented in the UI tree."""
    source_path = str(complete_media_structure)

    # Scan and group
    scan_results = scan_directory(source_path)
    video_files = [r.path for r in scan_results if r.is_video]
    groups = build_media_groups(video_files, source_path)

    # Simulate tree structure
    tree_structure: dict[str, list[dict[str, Any]]]
    tree_structure = {"Movies": [], "Shows": [], "Unknown": []}

    for group in groups:
        dir_name = Path(group.directory).name

        if group.kind == "movie":
            tree_structure["Movies"].append(
                {
                    "name": dir_name,
                    "directory": group.directory,
                    "video_count": len(group.primary_videos),
                }
            )
        elif group.kind == "show":
            tree_structure["Shows"].append(
                {
                    "name": dir_name,
                    "directory": group.directory,
                    "video_count": len(group.primary_videos),
                }
            )
        else:
            tree_structure["Unknown"].append(
                {
                    "name": dir_name,
                    "directory": group.directory,
                    "video_count": len(group.primary_videos),
                }
            )

    # Verify tree structure
    assert len(tree_structure["Movies"]) == 2
    assert len(tree_structure["Shows"]) == 1
    assert len(tree_structure["Unknown"]) == 0

    # Verify movie nodes
    movie_names = {node["name"] for node in tree_structure["Movies"]}
    assert "The Matrix (1999)" in movie_names
    assert "Inception (2010)" in movie_names

    # Verify show node
    show_names = {node["name"] for node in tree_structure["Shows"]}
    assert "Breaking Bad" in show_names

    # Verify video counts
    show_node = tree_structure["Shows"][0]
    assert show_node["video_count"] == 3  # All episodes grouped together
