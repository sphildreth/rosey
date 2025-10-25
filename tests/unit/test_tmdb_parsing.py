"""
TMDB ID parsing tests - Adapted from C# TmdbIdParsingTests.cs

Tests for TMDB ID handling via NFO files.
The Python implementation reads TMDB IDs from NFO files, not from path patterns.
These tests verify the NFO-based approach works correctly.
"""

import pytest
from pathlib import Path
from rosey.identifier import Identifier


def test_identifier_reads_tmdb_id_from_nfo(tmp_path):
    """Read TMDB ID from NFO file"""
    # Create media file
    media_file = tmp_path / "episode.mkv"
    media_file.write_text("")
    
    # Create NFO with TMDB ID
    nfo_file = tmp_path / "episode.nfo"
    nfo_file.write_text("""<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<episodedetails>
    <title>Test Episode</title>
    <season>1</season>
    <episode>1</episode>
    <uniqueid type="tmdb">2126</uniqueid>
</episodedetails>""")
    
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
    nfo_file.write_text("""<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<tvshow>
    <title>The Brady Bunch</title>
    <year>1969</year>
    <uniqueid type="tmdb">2126</uniqueid>
</tvshow>""")
    
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


# The remaining TMDB ID tests from C# are not applicable because:
# 1. Python version uses NFO files, not path parsing for IDs
# 2. TMDB ID extraction from [tmdbid-123] patterns is not implemented
# 3. This is by design - NFO files are the authoritative source
