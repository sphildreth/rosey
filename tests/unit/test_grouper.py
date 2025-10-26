"""Tests for media grouping module."""

import pytest

from rosey.grouper import MediaGroup, build_media_groups, classify_group, get_media_directory


@pytest.fixture
def temp_movie_structure(tmp_path):
    """Create a movie structure."""
    movie_dir = tmp_path / "The Matrix (1999)"
    movie_dir.mkdir()

    (movie_dir / "The Matrix (1999).mkv").write_text("video")
    (movie_dir / "The Matrix (1999).srt").write_text("subtitle")
    (movie_dir / "movie.nfo").write_text(
        """<?xml version="1.0"?>
<movie>
    <title>The Matrix</title>
    <year>1999</year>
    <tmdbid>603</tmdbid>
</movie>"""
    )

    return tmp_path, movie_dir


@pytest.fixture
def temp_show_structure(tmp_path):
    """Create a TV show structure."""
    show_dir = tmp_path / "Breaking Bad"
    season1 = show_dir / "Season 01"
    season2 = show_dir / "Season 02"

    season1.mkdir(parents=True)
    season2.mkdir(parents=True)

    (season1 / "Breaking Bad - S01E01.mkv").write_text("video")
    (season1 / "Breaking Bad - S01E02.mkv").write_text("video")
    (season1 / "Breaking Bad - S01E01.srt").write_text("subtitle")

    (season2 / "Breaking Bad - S02E01.mkv").write_text("video")

    (show_dir / "tvshow.nfo").write_text(
        """<?xml version="1.0"?>
<tvshow>
    <title>Breaking Bad</title>
    <year>2008</year>
    <tmdbid>1396</tmdbid>
</tvshow>"""
    )

    return tmp_path, show_dir


@pytest.fixture
def temp_mixed_structure(tmp_path):
    """Create a mixed content structure."""
    mixed_dir = tmp_path / "Mixed"
    mixed_dir.mkdir()

    (mixed_dir / "Movie1.mkv").write_text("video")
    (mixed_dir / "Movie2.mkv").write_text("video")

    return tmp_path, mixed_dir


def test_get_media_directory_movie(temp_movie_structure):
    """Test media directory detection for a movie."""
    tmp_path, movie_dir = temp_movie_structure
    video_path = str(movie_dir / "The Matrix (1999).mkv")

    media_dir = get_media_directory(video_path, str(tmp_path))

    assert media_dir == str(movie_dir)


def test_get_media_directory_show(temp_show_structure):
    """Test media directory detection for a TV show."""
    tmp_path, show_dir = temp_show_structure
    video_path = str(show_dir / "Season 01" / "Breaking Bad - S01E01.mkv")

    media_dir = get_media_directory(video_path, str(tmp_path))

    # Should return the show directory, not the season folder
    assert media_dir == str(show_dir)


def test_build_media_groups_movie(temp_movie_structure):
    """Test building media groups for a movie."""
    tmp_path, movie_dir = temp_movie_structure
    video_path = str(movie_dir / "The Matrix (1999).mkv")

    groups = build_media_groups([video_path], str(tmp_path))

    assert len(groups) == 1
    group = groups[0]

    assert group.directory == str(movie_dir)
    assert len(group.primary_videos) == 1
    assert group.primary_videos[0] == video_path
    assert group.kind == "movie"


def test_build_media_groups_show(temp_show_structure):
    """Test building media groups for a TV show."""
    tmp_path, show_dir = temp_show_structure

    video_paths = [
        str(show_dir / "Season 01" / "Breaking Bad - S01E01.mkv"),
        str(show_dir / "Season 01" / "Breaking Bad - S01E02.mkv"),
        str(show_dir / "Season 02" / "Breaking Bad - S02E01.mkv"),
    ]

    groups = build_media_groups(video_paths, str(tmp_path))

    assert len(groups) == 1
    group = groups[0]

    assert group.directory == str(show_dir)
    assert len(group.primary_videos) == 3
    assert group.kind == "show"


def test_build_media_groups_mixed_content_relaxed(temp_mixed_structure):
    """Test mixed content with relaxed enforcement."""
    tmp_path, mixed_dir = temp_mixed_structure

    video_paths = [
        str(mixed_dir / "Movie1.mkv"),
        str(mixed_dir / "Movie2.mkv"),
    ]

    groups = build_media_groups(video_paths, str(tmp_path), enforce_one_media=False)

    assert len(groups) == 1
    group = groups[0]

    assert group.directory == str(mixed_dir)
    assert len(group.primary_videos) == 2
    # Without clear classification, should be unknown or heuristically classified
    assert group.kind in ["unknown", "show"]


def test_build_media_groups_mixed_content_strict(temp_mixed_structure):
    """Test mixed content with strict enforcement."""
    tmp_path, mixed_dir = temp_mixed_structure

    video_paths = [
        str(mixed_dir / "Movie1.mkv"),
        str(mixed_dir / "Movie2.mkv"),
    ]

    groups = build_media_groups(video_paths, str(tmp_path), enforce_one_media=True)

    assert len(groups) == 1
    group = groups[0]

    assert group.kind == "unknown"
    assert len(group.errors) > 0


def test_discover_companions(temp_movie_structure):
    """Test companion file discovery."""
    tmp_path, movie_dir = temp_movie_structure
    video_path = str(movie_dir / "The Matrix (1999).mkv")

    groups = build_media_groups([video_path], str(tmp_path))
    group = groups[0]

    # Should discover the subtitle
    assert "The Matrix (1999)" in group.companions
    companions = group.companions["The Matrix (1999)"]
    assert any("srt" in str(c) for c in companions)

    # Should discover the directory-level NFO
    assert len(group.directory_companions) > 0
    assert any("nfo" in str(c) for c in group.directory_companions)


def test_classify_group_movie_single_video(tmp_path):
    """Test classifying a single video as movie."""
    movie_dir = tmp_path / "Test Movie"
    movie_dir.mkdir()
    (movie_dir / "Test Movie.mkv").write_text("video")

    group = MediaGroup(str(movie_dir))
    group.primary_videos = [str(movie_dir / "Test Movie.mkv")]

    classify_group(group)

    assert group.kind == "movie"


def test_classify_group_show_with_episodes(tmp_path):
    """Test classifying files with episode patterns as show."""
    show_dir = tmp_path / "Test Show"
    show_dir.mkdir()
    (show_dir / "Test Show - S01E01.mkv").write_text("video")
    (show_dir / "Test Show - S01E02.mkv").write_text("video")

    group = MediaGroup(str(show_dir))
    group.primary_videos = [
        str(show_dir / "Test Show - S01E01.mkv"),
        str(show_dir / "Test Show - S01E02.mkv"),
    ]

    classify_group(group)

    assert group.kind == "show"


def test_classify_group_show_with_season_folders(tmp_path):
    """Test classifying with season folders as show."""
    show_dir = tmp_path / "Test Show"
    season1 = show_dir / "Season 01"
    season1.mkdir(parents=True)
    (season1 / "episode.mkv").write_text("video")

    group = MediaGroup(str(show_dir))
    group.primary_videos = [str(season1 / "episode.mkv")]

    classify_group(group)

    assert group.kind == "show"


def test_classify_group_with_nfo_movie(tmp_path):
    """Test classifying with NFO as movie."""
    movie_dir = tmp_path / "Test Movie"
    movie_dir.mkdir()
    (movie_dir / "video.mkv").write_text("video")

    group = MediaGroup(str(movie_dir))
    group.primary_videos = [str(movie_dir / "video.mkv")]
    group.nfo_data = {"tmdbid": "12345", "season": None}

    classify_group(group)

    assert group.kind == "movie"


def test_classify_group_with_nfo_show(tmp_path):
    """Test classifying with NFO as show."""
    show_dir = tmp_path / "Test Show"
    show_dir.mkdir()
    (show_dir / "video.mkv").write_text("video")

    group = MediaGroup(str(show_dir))
    group.primary_videos = [str(show_dir / "video.mkv")]
    group.nfo_data = {"tmdbid": "12345", "season": 1}

    classify_group(group)

    assert group.kind == "show"


def test_get_media_directory_generic_root(tmp_path):
    """Test that generic roots are skipped."""
    # Create structure: tmp_path/movies/Movie (1999)/video.mkv
    movies_dir = tmp_path / "movies"
    movie_dir = movies_dir / "Movie (1999)"
    movie_dir.mkdir(parents=True)
    video_path = str(movie_dir / "video.mkv")

    media_dir = get_media_directory(video_path, str(tmp_path))

    # Should return "Movie (1999)" directory, not "movies"
    assert media_dir == str(movie_dir)


def test_multiple_groups_same_scan(tmp_path):
    """Test building multiple groups from a single scan."""
    # Create two separate movie directories
    movie1 = tmp_path / "Movie1"
    movie2 = tmp_path / "Movie2"
    movie1.mkdir()
    movie2.mkdir()

    (movie1 / "video.mkv").write_text("video")
    (movie2 / "video.mkv").write_text("video")

    video_paths = [
        str(movie1 / "video.mkv"),
        str(movie2 / "video.mkv"),
    ]

    groups = build_media_groups(video_paths, str(tmp_path))

    assert len(groups) == 2

    group_dirs = {g.directory for g in groups}
    assert str(movie1) in group_dirs
    assert str(movie2) in group_dirs


def test_date_based_episode_classification(tmp_path):
    """Test that date-based episodes are classified as shows."""
    show_dir = tmp_path / "Daily Show"
    show_dir.mkdir()
    (show_dir / "Daily Show - 2023-01-15.mkv").write_text("video")

    group = MediaGroup(str(show_dir))
    group.primary_videos = [str(show_dir / "Daily Show - 2023-01-15.mkv")]

    classify_group(group)

    assert group.kind == "show"
