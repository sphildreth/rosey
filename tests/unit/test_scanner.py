"""Tests for scanner module."""

from pathlib import Path

import pytest

from rosey.scanner import Scanner, scan_directory


@pytest.fixture
def temp_media_tree(tmp_path):
    """Create a temporary media directory structure."""
    # Create some video files
    videos = [
        "The Matrix (1999).mkv",
        "Inception (2010).mp4",
        "Show S01E01.mkv",
        "Show S01E02.avi",
    ]

    for video in videos:
        (tmp_path / video).write_text("fake video data")

    # Create a subdirectory with more files
    subdir = tmp_path / "subfolder"
    subdir.mkdir()
    (subdir / "Movie.mp4").write_text("fake video")
    (subdir / "document.txt").write_text("not a video")

    return tmp_path


def test_scanner_basic(temp_media_tree):
    """Test basic scanning functionality."""
    scanner = Scanner(max_workers=2)
    results = scanner.scan(str(temp_media_tree))

    # Should find all video files
    video_results = [r for r in results if r.is_video]
    assert len(video_results) == 5

    # Check that all videos were found
    video_paths = {Path(r.path).name for r in video_results}
    assert "The Matrix (1999).mkv" in video_paths
    assert "Inception (2010).mp4" in video_paths
    assert "Show S01E01.mkv" in video_paths


def test_scanner_video_extensions(temp_media_tree):
    """Test that scanner recognizes video extensions."""
    scanner = Scanner()
    results = scanner.scan(str(temp_media_tree))

    # Check video detection
    for result in results:
        if result.path.endswith((".mkv", ".mp4", ".avi")):
            assert result.is_video
        elif result.path.endswith(".txt"):
            assert not result.is_video


def test_scanner_empty_directory(tmp_path):
    """Test scanning an empty directory."""
    scanner = Scanner()
    results = scanner.scan(str(tmp_path))

    assert results == []


def test_scanner_nonexistent_path():
    """Test scanning a nonexistent path."""
    scanner = Scanner()
    results = scanner.scan("/nonexistent/path/12345")

    assert results == []


def test_scanner_single_file(tmp_path):
    """Test scanning a single file instead of directory."""
    video_file = tmp_path / "single.mkv"
    video_file.write_text("fake video")

    scanner = Scanner()
    results = scanner.scan(str(video_file))

    assert len(results) == 1
    assert results[0].is_video
    assert results[0].path == str(video_file)


def test_scanner_concurrency(temp_media_tree):
    """Test scanner with different concurrency levels."""
    # With 1 worker
    scanner1 = Scanner(max_workers=1)
    results1 = scanner1.scan(str(temp_media_tree))

    # With 4 workers
    scanner4 = Scanner(max_workers=4)
    results4 = scanner4.scan(str(temp_media_tree))

    # Both should find the same files
    paths1 = {r.path for r in results1}
    paths4 = {r.path for r in results4}
    assert paths1 == paths4


def test_scan_directory_convenience(temp_media_tree):
    """Test convenience function."""
    results = scan_directory(str(temp_media_tree), max_workers=2)

    video_results = [r for r in results if r.is_video]
    assert len(video_results) == 5


def test_scanner_large_tree(tmp_path):
    """Test scanner with larger directory structure."""
    # Create nested structure
    for i in range(5):
        season_dir = tmp_path / f"Season {i:02d}"
        season_dir.mkdir()
        for j in range(10):
            (season_dir / f"Episode_{j:02d}.mkv").write_text("fake")

    scanner = Scanner(max_workers=4)
    results = scanner.scan(str(tmp_path))

    video_results = [r for r in results if r.is_video]
    assert len(video_results) == 50


def test_scanner_file_sizes(temp_media_tree):
    """Test that scanner gets file sizes for videos."""
    scanner = Scanner()
    results = scanner.scan(str(temp_media_tree))

    video_results = [r for r in results if r.is_video]

    # All videos should have size > 0
    for result in video_results:
        assert result.size_bytes > 0
