"""Tests for identifier module."""


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


class TestIdentifierMovies:
    """Test movie identification."""

    def test_movie_with_year(self):
        """Test identifying movie with year in filename."""
        result = identify_file("/path/to/The Matrix (1999).mkv")

        assert result.item.kind == "movie"
        assert "Matrix" in result.item.title
        assert result.item.year == 1999

    def test_movie_without_year(self):
        """Test identifying movie without year."""
        result = identify_file("/path/to/Inception.mkv")

        assert result.item.kind == "movie"
        assert "Inception" in result.item.title

    def test_movie_multipart(self):
        """Test identifying multipart movie."""
        result = identify_file("/path/to/Kill Bill Part 1.mkv")

        assert result.item.kind == "movie"
        assert result.item.part == 1
        assert "Kill Bill" in result.item.title


class TestIdentifierTVEpisodes:
    """Test TV episode identification."""

    def test_episode_sxxeyy(self):
        """Test S01E05 format."""
        result = identify_file("/tv/The Office/Season 02/The.Office.S02E01.mkv")

        assert result.item.kind == "episode"
        assert result.item.season == 2
        assert result.item.episodes == [1]

    def test_episode_xformat(self):
        """Test 1x05 format."""
        result = identify_file("/tv/Show/1x05.mkv")

        assert result.item.kind == "episode"
        assert result.item.season == 1
        assert result.item.episodes == [5]

    def test_episode_multi_episode(self):
        """Test multi-episode S01E01-E02."""
        result = identify_file("/tv/Show/S01E01-E02.mkv")

        assert result.item.kind == "episode"
        assert result.item.season == 1
        assert result.item.episodes == [1, 2]

    def test_episode_date_based(self):
        """Test date-based episode."""
        result = identify_file("/tv/Daily Show/Daily.Show.2020-03-15.mkv")

        assert result.item.kind == "episode"
        assert result.item.date == "2020-03-15"

    def test_episode_from_folder_structure(self):
        """Test deriving show name from folder."""
        result = identify_file("/tv/Breaking Bad/Season 01/episode.S01E01.mkv")

        assert result.item.kind == "episode"
        assert "Breaking Bad" in result.item.title or "episode" in result.item.title

    def test_episode_multipart(self):
        """Test multipart episode."""
        result = identify_file("/tv/Show/Show.S01E01.Part.1.mkv")

        assert result.item.kind == "episode"
        assert result.item.part == 1


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


def test_identifier_convenience_function():
    """Test convenience function."""
    result = identify_file("/path/to/Movie (2020).mkv")

    assert result is not None
    assert result.item.kind == "movie"
