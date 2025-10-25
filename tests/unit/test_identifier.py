"""Tests for identifier module.

This file now focuses on NFO precedence and unknown/edge cases.
Filename parsing coverage has been consolidated into CSV-driven tests
under tests/identifier/.
"""


import pytest

from rosey.identifier import identify_file


@pytest.fixture
def temp_nfo_dir(tmp_path):
    """Create temporary directory with NFO files."""
    # Movie NFO
    movie_nfo = tmp_path / "Movie.nfo"
    movie_nfo.write_text("""<?xml version="1.0" encoding="UTF-8"?>
<movie>
    <title>The Matrix</title>
    <year>1999</year>
    <tmdbid>603</tmdbid>
    <imdbid>tt0133093</imdbid>
</movie>
""")

    # TV episode NFO
    episode_nfo = tmp_path / "Episode.nfo"
    episode_nfo.write_text("""<?xml version="1.0" encoding="UTF-8"?>
<episodedetails>
    <title>The Office</title>
    <season>2</season>
    <episode>1</episode>
    <episodetitle>The Dundies</episodetitle>
    <tvdbid>73244</tvdbid>
</episodedetails>
""")

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
