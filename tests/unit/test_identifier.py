"""Tests for identifier module.

This file now focuses on NFO precedence and unknown/edge cases.
Filename parsing coverage has been consolidated into CSV-driven tests
under tests/identifier/.
"""

from unittest.mock import patch

import pytest

from rosey.identifier import Identifier, identify_file


@pytest.fixture
def temp_nfo_dir(tmp_path):
    """Create temporary directory with NFO files."""
    # Movie NFO
    movie_nfo = tmp_path / "Movie.nfo"
    movie_nfo.write_text(
        """<?xml version="1.0" encoding="UTF-8"?>
<movie>
    <title>The Matrix</title>
    <year>1999</year>
    <tmdbid>603</tmdbid>
    <imdbid>tt0133093</imdbid>
</movie>
"""
    )

    # TV episode NFO
    episode_nfo = tmp_path / "Episode.nfo"
    episode_nfo.write_text(
        """<?xml version="1.0" encoding="UTF-8"?>
<episodedetails>
    <title>The Office</title>
    <season>2</season>
    <episode>1</episode>
    <episodetitle>The Dundies</episodetitle>
    <tvdbid>73244</tvdbid>
</episodedetails>
"""
    )

    return tmp_path


class TestIdentifierWithNFO:
    """Test identification with NFO files."""

    def test_movie_nfo(self, temp_nfo_dir):
        """Test movie identification with NFO."""
        video_file = temp_nfo_dir / "Movie.mkv"
        video_file.write_text("fake video")

        result = identify_file(str(video_file))

        assert result.item.kind == "movie"
        assert result.item.title == "The Matrix"
        assert result.item.year == 1999
        assert "tmdbid" in result.item.nfo
        assert result.item.nfo["tmdbid"] == "603"

    def test_episode_nfo(self, temp_nfo_dir):
        """Test episode identification with NFO."""
        video_file = temp_nfo_dir / "Episode.mkv"
        video_file.write_text("fake video")

        result = identify_file(str(video_file))

        assert result.item.kind == "episode"
        assert result.item.season == 2
        assert result.item.episodes == [1]
        assert "episode_title" in result.item.nfo


class TestIdentifierUnknown:
    """Test unknown media identification."""

    def test_unknown_no_pattern(self):
        """Test file with no recognizable pattern."""
        result = identify_file("/path/to/random_file.mkv")

        # Could be unknown or movie depending on implementation
        assert result.item.kind in ["unknown", "movie"]

    def test_unknown_ambiguous(self):
        """Test ambiguous filename."""
        result = identify_file("/path/to/abc.mkv")
        assert result.item is not None


class TestEpisodeTitleExtraction:
    """Test episode title extraction from filenames."""

    def test_episode_title_with_dash(self):
        """Test episode title extraction with dash separator."""
        result = identify_file("/path/to/Show S01E01 - Pilot.mkv")
        assert result.item.kind == "episode"
        assert result.item.nfo.get("episode_title") == "Pilot"

    def test_episode_title_with_parentheses(self):
        """Test episode title extraction with parentheses."""
        result = identify_file("/path/to/Rhoda S04E20 (Brenda and the Bank Girl).mkv")
        assert result.item.kind == "episode"
        # Note: Rhoda alone may not be recognized as a title
        assert result.item.season == 4
        assert result.item.episodes == [20]
        assert result.item.nfo.get("episode_title") == "Brenda and the Bank Girl"

    def test_episode_title_with_quality_tags(self):
        """Test episode title extraction with quality tags that should be removed."""
        result = identify_file("/path/to/Show S02E03 - Episode Name [1080p].mkv")
        assert result.item.nfo.get("episode_title") == "Episode Name"

    def test_episode_no_title(self):
        """Test episode without a title."""
        result = identify_file("/path/to/Show S01E05.mkv")
        assert result.item.kind == "episode"
        # episode_title should not be present or should be None
        assert not result.item.nfo.get("episode_title")

    def test_episode_title_nfo_precedence(self, temp_nfo_dir):
        """Test that NFO episode title takes precedence over filename."""
        video_file = temp_nfo_dir / "Episode.mkv"
        video_file.write_text("fake video")

        result = identify_file(str(video_file))
        # NFO has episodetitle "The Dundies", should use that
        assert result.item.nfo.get("episode_title") == "The Dundies"


class TestMoviesAlwaysInOwnDirectory:
    """Test movies_always_in_own_directory configuration option."""

    def test_movie_in_show_folder_prevented(self, tmp_path):
        """Test that movies in show folders are prevented when option is enabled."""
        # Create a show folder structure
        show_dir = tmp_path / "The Office"
        show_dir.mkdir()
        season_dir = show_dir / "Season 1"
        season_dir.mkdir()
        movie_file = season_dir / "The.Matrix.1999.mkv"
        movie_file.write_text("fake movie")

        # Create identifier with option enabled
        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "unknown"
        assert any("show folder" in reason.lower() for reason in result.reasons)

    def test_movie_with_multiple_files_prevented(self, tmp_path):
        """Test that movies with other media files in directory are prevented."""
        # Create directory with multiple media files
        movie_dir = tmp_path / "Movies"
        movie_dir.mkdir()
        movie_file = movie_dir / "The.Matrix.1999.mkv"
        movie_file.write_text("fake movie")
        extra_file = movie_dir / "extra.mkv"
        extra_file.write_text("fake extra")

        # Create identifier with option enabled
        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "unknown"
        assert any("show folder" in reason.lower() for reason in result.reasons)

    def test_movie_alone_allowed(self, tmp_path):
        """Test that movies alone in directory are allowed when option is enabled."""
        # Create directory with single movie file
        movie_dir = tmp_path / "Movies"
        movie_dir.mkdir()
        movie_file = movie_dir / "The.Matrix.1999.mkv"
        movie_file.write_text("fake movie")
        # Add a subdirectory (should be allowed)
        subdir = movie_dir / "extras"
        subdir.mkdir()

        # Create identifier with option enabled
        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "movie"
        assert result.item.title == "The Matrix"
        assert result.item.year == 1999

    def test_option_disabled_allows_anywhere(self, tmp_path):
        """Test that when option is disabled, movies can be anywhere."""
        # Create a show folder structure
        show_dir = tmp_path / "The Office"
        show_dir.mkdir()
        season_dir = show_dir / "Season 1"
        season_dir.mkdir()
        movie_file = season_dir / "The.Matrix.1999.mkv"
        movie_file.write_text("fake movie")
        extra_file = season_dir / "extra.mkv"
        extra_file.write_text("fake extra")

        # Create identifier with option disabled (default)
        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = False
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        # Should still be classified as movie despite being in show folder with other files
        assert result.item.kind == "movie"


class TestMinimumMovieDuration:
    """Test minimum_movie_duration_minutes configuration option."""

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_long_movie_accepted(self, mock_duration):
        """Test that movies longer than minimum duration are accepted."""
        mock_duration.return_value = 120  # 120 minutes, above default 60

        result = identify_file("/path/to/long_movie.mkv")

        assert result.item.kind == "movie"
        assert any("meets minimum" in reason.lower() for reason in result.reasons)

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_short_movie_rejected(self, mock_duration):
        """Test that movies shorter than minimum duration are rejected."""
        mock_duration.return_value = 30  # 30 minutes, below default 60

        result = identify_file("/path/to/short_clip.mkv")

        assert result.item.kind == "unknown"
        assert any(
            "too short" in reason.lower() or "not a movie" in reason.lower()
            for reason in result.reasons
        )

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_exact_minimum_duration_accepted(self, mock_duration):
        """Test that movies exactly at minimum duration are accepted."""
        mock_duration.return_value = 60  # Exactly 60 minutes

        result = identify_file("/path/to/exact_movie.mkv")

        assert result.item.kind == "movie"
        assert any("meets minimum" in reason.lower() for reason in result.reasons)

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_custom_minimum_duration(self, mock_duration):
        """Test custom minimum duration setting."""
        mock_duration.return_value = 90  # 90 minutes

        # Create identifier with custom minimum duration
        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.minimum_movie_duration_minutes = 100  # Require 100 minutes
        identifier = Identifier(config=config)

        result = identifier.identify("/path/to/movie.mkv")

        assert result.item.kind == "unknown"  # 90 < 100, so rejected
        assert any("not a movie" in reason.lower() for reason in result.reasons)

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_duration_check_failure_fallback(self, mock_duration):
        """Test that duration check failure falls back to normal movie classification."""
        mock_duration.return_value = None  # Simulate ffprobe failure

        result = identify_file("/path/to/movie.mkv")

        # Should still classify as movie when duration check fails
        assert result.item.kind == "movie"

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_zero_duration_handled(self, mock_duration):
        """Test handling of zero duration videos."""
        mock_duration.return_value = 0  # 0 minutes

        result = identify_file("/path/to/zero_duration.mkv")

        assert result.item.kind == "unknown"
        assert any("not a movie" in reason.lower() for reason in result.reasons)

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_negative_duration_handled(self, mock_duration):
        """Test handling of negative duration (shouldn't happen but be safe)."""
        mock_duration.return_value = -5  # Negative minutes

        result = identify_file("/path/to/negative_duration.mkv")

        assert result.item.kind == "unknown"
        assert any("not a movie" in reason.lower() for reason in result.reasons)

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_duration_with_nfo_title(self, mock_duration, tmp_path):
        """Test duration checking with NFO title."""
        mock_duration.return_value = 30  # Short duration

        # Create NFO file
        nfo_file = tmp_path / "movie.nfo"
        nfo_file.write_text(
            """<?xml version="1.0" encoding="UTF-8"?>
<movie>
    <title>The Matrix</title>
    <year>1999</year>
</movie>
"""
        )
        movie_file = tmp_path / "movie.mkv"
        movie_file.write_text("fake movie")

        result = identify_file(str(movie_file))

        # Should be classified as unknown due to short duration, even with NFO
        assert result.item.kind == "unknown"
        assert any("not a movie" in reason.lower() for reason in result.reasons)

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_duration_with_year_filename(self, mock_duration):
        """Test duration checking with year in filename."""
        mock_duration.return_value = 45  # Below minimum

        result = identify_file("/path/to/Movie.2020.mkv")

        assert result.item.kind == "unknown"
        assert any("not a movie" in reason.lower() for reason in result.reasons)

    @patch("rosey.identifier.Identifier._get_video_duration_minutes")
    def test_duration_with_part_filename(self, mock_duration):
        """Test duration checking with part number in filename."""
        mock_duration.return_value = 45  # Below minimum

        result = identify_file("/path/to/Movie.Part.1.mkv")

        assert result.item.kind == "unknown"
        assert any("not a movie" in reason.lower() for reason in result.reasons)


class TestMoviesAlwaysInOwnDirectoryEdgeCases:
    """Test edge cases for movies_always_in_own_directory configuration."""

    def test_season_folder_detection(self, tmp_path):
        """Test detection of season folders."""
        # Create season folder structure
        show_dir = tmp_path / "Show"
        show_dir.mkdir()
        season_dir = show_dir / "Season 01"
        season_dir.mkdir()
        movie_file = season_dir / "movie.mkv"
        movie_file.write_text("fake movie")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "unknown"
        assert any("show folder" in reason.lower() for reason in result.reasons)

    def test_season_folder_sxx_format(self, tmp_path):
        """Test detection of season folders in SXX format."""
        show_dir = tmp_path / "Show"
        show_dir.mkdir()
        season_dir = show_dir / "S02"
        season_dir.mkdir()
        movie_file = season_dir / "movie.mkv"
        movie_file.write_text("fake movie")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "unknown"
        assert any("show folder" in reason.lower() for reason in result.reasons)

    def test_multiple_season_subdirs(self, tmp_path):
        """Test detection when directory contains multiple season subdirectories."""
        show_dir = tmp_path / "Show"
        show_dir.mkdir()
        season1_dir = show_dir / "Season 1"
        season1_dir.mkdir()
        season2_dir = show_dir / "Season 2"
        season2_dir.mkdir()
        movie_file = show_dir / "movie.mkv"  # Movie in show root
        movie_file.write_text("fake movie")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "unknown"
        assert any("show folder" in reason.lower() for reason in result.reasons)

    def test_directory_with_many_episodes(self, tmp_path):
        """Test detection when directory contains many episode files."""
        show_dir = tmp_path / "Show"
        show_dir.mkdir()
        # Create multiple episode files
        for i in range(5):
            episode_file = show_dir / f"episode_{i}.mkv"
            episode_file.write_text("fake episode")
        movie_file = show_dir / "movie.mkv"
        movie_file.write_text("fake movie")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "unknown"
        assert any("show folder" in reason.lower() for reason in result.reasons)

    def test_subdirectory_allowed(self, tmp_path):
        """Test that subdirectories are allowed in movie directories."""
        movie_dir = tmp_path / "Movie"
        movie_dir.mkdir()
        movie_file = movie_dir / "movie.mkv"
        movie_file.write_text("fake movie")
        # Create subdirectories (should be allowed)
        subs_dir = movie_dir / "subs"
        subs_dir.mkdir()
        extras_dir = movie_dir / "extras"
        extras_dir.mkdir()
        # Add files to subdirectories (should be allowed)
        sub_file = subs_dir / "subtitles.srt"
        sub_file.write_text("fake subs")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "movie"

    def test_non_media_files_allowed(self, tmp_path):
        """Test that non-media files are allowed in movie directories."""
        movie_dir = tmp_path / "Movie"
        movie_dir.mkdir()
        movie_file = movie_dir / "movie.mkv"
        movie_file.write_text("fake movie")
        # Add non-media files (should be allowed)
        nfo_file = movie_dir / "movie.nfo"
        nfo_file.write_text("fake nfo")
        txt_file = movie_dir / "readme.txt"
        txt_file.write_text("fake readme")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "movie"

    def test_different_media_extensions(self, tmp_path):
        """Test detection with different media file extensions."""
        movie_dir = tmp_path / "Movie"
        movie_dir.mkdir()
        movie_file = movie_dir / "movie.mkv"
        movie_file.write_text("fake movie")
        # Add different media extensions
        mp4_file = movie_dir / "extra.mp4"
        mp4_file.write_text("fake mp4")
        avi_file = movie_dir / "extra.avi"
        avi_file.write_text("fake avi")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "unknown"
        assert any("show folder" in reason.lower() for reason in result.reasons)

    def test_empty_directory(self, tmp_path):
        """Test movie in completely empty directory."""
        movie_dir = tmp_path / "Movie"
        movie_dir.mkdir()
        movie_file = movie_dir / "movie.mkv"
        movie_file.write_text("fake movie")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "movie"

    def test_movie_in_root_directory(self, tmp_path):
        """Test movie in root directory (no parent folder structure)."""
        movie_file = tmp_path / "movie.mkv"
        movie_file.write_text("fake movie")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        # Root directory with single file should be allowed
        assert result.item.kind == "movie"

    def test_movie_in_generic_folder(self, tmp_path):
        """Test movie in generic folder (not detected as show folder)."""
        generic_dir = tmp_path / "videos"
        generic_dir.mkdir()
        movie_file = generic_dir / "movie.mkv"
        movie_file.write_text("fake movie")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "movie"

    def test_case_insensitive_season_detection(self, tmp_path):
        """Test case insensitive season folder detection."""
        show_dir = tmp_path / "Show"
        show_dir.mkdir()
        season_dir = show_dir / "season 1"
        season_dir.mkdir()
        movie_file = season_dir / "movie.mkv"
        movie_file.write_text("fake movie")

        from rosey.config import RoseyConfig

        config = RoseyConfig()
        config.identification.movies_always_in_own_directory = True
        identifier = Identifier(config=config)

        result = identifier.identify(str(movie_file))

        assert result.item.kind == "unknown"
        assert any("show folder" in reason.lower() for reason in result.reasons)
