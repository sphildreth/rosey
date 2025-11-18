"""
TMDB ID parsing tests - Adapted from C# TmdbIdParsingTests.cs

Tests for TMDB ID handling via NFO files and directory paths.
The Python implementation reads TMDB IDs from:
1. NFO files (preferred)
2. Directory path patterns [tmdbid-XXX]
"""

from unittest.mock import Mock

from rosey.identifier import Identifier
from rosey.identifier.patterns import extract_tmdb_id_from_path


def test_identifier_reads_tmdb_id_from_nfo(tmp_path):
    """Read TMDB ID from NFO file"""
    # Create media file
    media_file = tmp_path / "episode.mkv"
    media_file.write_text("")

    # Create NFO with TMDB ID
    nfo_file = tmp_path / "episode.nfo"
    nfo_file.write_text(
        """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<episodedetails>
    <title>Test Episode</title>
    <season>1</season>
    <episode>1</episode>
    <uniqueid type="tmdb">2126</uniqueid>
</episodedetails>"""
    )

    identifier = Identifier()
    result = identifier.identify(str(media_file))

    assert result.item.nfo.get("tmdbid") == "2126"
    assert result.item.season == 1
    assert result.item.episodes == [1]


def test_identifier_reads_tmdb_id_from_tvshow_nfo(tmp_path):
    """Read TMDB ID from tvshow.nfo file"""
    show_dir = tmp_path / "The Brady Bunch (1969)"
    show_dir.mkdir()

    # Create tvshow.nfo with TMDB ID
    nfo_file = show_dir / "tvshow.nfo"
    nfo_file.write_text(
        """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<tvshow>
    <title>The Brady Bunch</title>
    <year>1969</year>
    <uniqueid type="tmdb">2126</uniqueid>
</tvshow>"""
    )

    # Create episode file
    season_dir = show_dir / "Season 01"
    season_dir.mkdir()
    media_file = season_dir / "S01E01.mkv"
    media_file.write_text("")

    identifier = Identifier()
    result = identifier.identify(str(media_file))

    # Episode should be identified even without episode NFO
    assert result.item.kind == "episode"
    assert result.item.season == 1
    assert result.item.episodes == [1]


def test_identifier_prefers_nfo_over_filename():
    """NFO data takes precedence over filename parsing"""
    # This is a design principle - NFO is authoritative
    identifier = Identifier(prefer_nfo=True)
    assert identifier.prefer_nfo is True


def test_extract_tmdb_id_from_movie_path():
    """Extract TMDB ID from movie folder name"""
    path = "/movies/The Matrix (1999) [tmdbid-603]/The Matrix.mkv"
    tmdb_id = extract_tmdb_id_from_path(path)
    assert tmdb_id == "603"


def test_extract_tmdb_id_from_tv_path():
    """Extract TMDB ID from TV show folder name"""
    path = "/tv/Breaking Bad (2008) [tmdbid-1396]/Season 01/S01E01.mkv"
    tmdb_id = extract_tmdb_id_from_path(path)
    assert tmdb_id == "1396"


def test_extract_tmdb_id_from_path_no_match():
    """Return None when no TMDB ID pattern exists"""
    path = "/movies/Random Movie/movie.mkv"
    tmdb_id = extract_tmdb_id_from_path(path)
    assert tmdb_id is None


def test_extract_tmdb_id_case_insensitive():
    """TMDB ID pattern should be case-insensitive"""
    path1 = "/movies/Movie [TMDBID-123]/movie.mkv"
    path2 = "/movies/Movie [TmDbId-456]/movie.mkv"
    assert extract_tmdb_id_from_path(path1) == "123"
    assert extract_tmdb_id_from_path(path2) == "456"


def test_tmdb_api_identifies_movie(tmp_path):
    """Use TMDB API to definitively identify a movie"""
    # Create folder with TMDB ID
    movie_dir = tmp_path / "The Matrix (1999) [tmdbid-603]"
    movie_dir.mkdir()
    media_file = movie_dir / "The Matrix.mkv"
    media_file.write_text("")

    # Mock TMDB provider
    mock_provider = Mock()
    mock_provider.get_movie_by_id.return_value = {"id": 603, "title": "The Matrix"}

    identifier = Identifier(tmdb_provider=mock_provider)
    result = identifier.identify(str(media_file))

    # Should be identified as movie due to TMDB API
    assert result.item.kind == "movie"
    assert result.item.nfo.get("tmdbid") == "603"
    mock_provider.get_movie_by_id.assert_called_once_with("603")
    assert any("definitive movie" in r.lower() for r in result.reasons)


def test_tmdb_api_identifies_tv_show(tmp_path):
    """Use TMDB API to definitively identify a TV show"""
    # Create folder with TMDB ID
    show_dir = tmp_path / "Breaking Bad (2008) [tmdbid-1396]"
    show_dir.mkdir()
    season_dir = show_dir / "Season 01"
    season_dir.mkdir()
    media_file = season_dir / "S01E01.mkv"
    media_file.write_text("")

    # Mock TMDB provider
    mock_provider = Mock()
    mock_provider.get_movie_by_id.return_value = None  # Not a movie
    mock_provider.get_tv_by_id.return_value = {"id": 1396, "name": "Breaking Bad"}

    identifier = Identifier(tmdb_provider=mock_provider)
    result = identifier.identify(str(media_file))

    # Should be identified as episode due to TMDB API
    assert result.item.kind == "episode"
    assert result.item.nfo.get("tmdbid") == "1396"
    mock_provider.get_movie_by_id.assert_called_once_with("1396")
    mock_provider.get_tv_by_id.assert_called_once_with("1396")
    assert any("definitive tv show" in r.lower() for r in result.reasons)


def test_tmdb_id_without_provider(tmp_path):
    """Gracefully handle TMDB ID when no provider available"""
    # Create folder with TMDB ID
    movie_dir = tmp_path / "The Matrix (1999) [tmdbid-603]"
    movie_dir.mkdir()
    media_file = movie_dir / "The Matrix.mkv"
    media_file.write_text("")

    # No provider passed
    identifier = Identifier()
    result = identifier.identify(str(media_file))

    # Should still identify using heuristics (has year, in own directory)
    assert result.item.kind == "movie"
    # TMDB ID won't be in NFO since we didn't query the API
    assert result.item.nfo.get("tmdbid") is None


def test_tmdb_api_caches_results(tmp_path):
    """TMDB API results should be cached to avoid duplicate queries"""
    # Create two files in same folder with TMDB ID
    movie_dir = tmp_path / "The Matrix (1999) [tmdbid-603]"
    movie_dir.mkdir()
    media_file1 = movie_dir / "The Matrix.mkv"
    media_file1.write_text("")
    media_file2 = movie_dir / "The Matrix - Extras.mkv"
    media_file2.write_text("")

    # Mock TMDB provider
    mock_provider = Mock()
    mock_provider.get_movie_by_id.return_value = {"id": 603, "title": "The Matrix"}

    identifier = Identifier(tmdb_provider=mock_provider)

    # Identify first file
    result1 = identifier.identify(str(media_file1))
    assert result1.item.kind == "movie"

    # Identify second file
    result2 = identifier.identify(str(media_file2))
    assert result2.item.kind == "movie"

    # TMDB API should only be called once due to caching
    assert mock_provider.get_movie_by_id.call_count == 1
    assert any("cached" in r.lower() for r in result2.reasons)
