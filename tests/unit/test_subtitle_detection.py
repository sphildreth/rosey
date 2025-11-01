"""Tests for subtitle file detection and language handling."""

import sys
from pathlib import Path

# Add src to path for testing
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from rosey.ui.main_window import (
    SUBTITLE_EXTENSIONS,
    extract_language_from_companion_filename,
    get_language_code_from_name,
    is_subtitle_file,
)


def test_subtitle_extensions_complete():
    """Test that all subtitle formats are recognized."""
    assert ".srt" in SUBTITLE_EXTENSIONS  # SubRip Text
    assert ".ssa" in SUBTITLE_EXTENSIONS  # SubStation Alpha
    assert ".ass" in SUBTITLE_EXTENSIONS  # Advanced SubStation Alpha
    assert ".vtt" in SUBTITLE_EXTENSIONS  # WebVTT
    assert ".sub" in SUBTITLE_EXTENSIONS  # VobSub / SubViewer / MicroDVD
    assert ".idx" in SUBTITLE_EXTENSIONS  # VobSub index
    assert ".sbv" in SUBTITLE_EXTENSIONS  # SubViewer
    assert ".lrc" in SUBTITLE_EXTENSIONS  # LRC (Lyric file)
    assert ".smi" in SUBTITLE_EXTENSIONS  # SAMI format
    assert ".stl" in SUBTITLE_EXTENSIONS  # Spruce subtitle file


def test_is_subtitle_file():
    """Test subtitle file detection."""
    # Positive cases - all subtitle formats
    assert is_subtitle_file("movie.srt")
    assert is_subtitle_file("show.ssa")
    assert is_subtitle_file("episode.ass")
    assert is_subtitle_file("video.vtt")
    assert is_subtitle_file("film.sub")
    assert is_subtitle_file("content.idx")
    assert is_subtitle_file("media.sbv")
    assert is_subtitle_file("song.lrc")
    assert is_subtitle_file("clip.smi")
    assert is_subtitle_file("feature.stl")

    # Case insensitive
    assert is_subtitle_file("MOVIE.SRT")
    assert is_subtitle_file("Show.Ass")

    # Negative cases
    assert not is_subtitle_file("movie.mkv")
    assert not is_subtitle_file("poster.jpg")
    assert not is_subtitle_file("movie.nfo")
    assert not is_subtitle_file("fanart.png")


def test_extract_language_from_filename_explicit():
    """Test language extraction from filename with explicit language."""
    # Full language names
    assert extract_language_from_companion_filename("English.srt") == "english"
    assert extract_language_from_companion_filename("Spanish.ass") == "spanish"
    assert extract_language_from_companion_filename("French.vtt") == "french"
    assert extract_language_from_companion_filename("German.sub") == "german"

    # Language codes
    assert extract_language_from_companion_filename("en.srt") == "en"
    assert extract_language_from_companion_filename("es.ass") == "es"
    assert extract_language_from_companion_filename("fr.vtt") == "fr"

    # Abbreviations
    assert extract_language_from_companion_filename("eng.srt") == "eng"
    assert extract_language_from_companion_filename("esp.ass") == "esp"

    # With prefixes
    assert extract_language_from_companion_filename("English.forced.srt") == "english"
    assert extract_language_from_companion_filename("Spanish_sdh.ass") == "spanish"


def test_extract_language_subtitle_default():
    """Test that subtitle files default to 'english' when no language detected."""
    # Subtitle files without language should default to "english"
    assert extract_language_from_companion_filename("movie.srt", filepath="movie.srt") == "english"
    assert extract_language_from_companion_filename("show.ass", filepath="show.ass") == "english"
    assert (
        extract_language_from_companion_filename("episode.vtt", filepath="episode.vtt") == "english"
    )
    assert extract_language_from_companion_filename("film.sub", filepath="film.sub") == "english"
    assert extract_language_from_companion_filename("video.idx", filepath="video.idx") == "english"
    assert (
        extract_language_from_companion_filename("content.sbv", filepath="content.sbv") == "english"
    )
    assert extract_language_from_companion_filename("media.lrc", filepath="media.lrc") == "english"
    assert extract_language_from_companion_filename("clip.smi", filepath="clip.smi") == "english"
    assert (
        extract_language_from_companion_filename("feature.stl", filepath="feature.stl") == "english"
    )
    assert (
        extract_language_from_companion_filename("subtitle.ssa", filepath="subtitle.ssa")
        == "english"
    )


def test_extract_language_non_subtitle_no_default():
    """Test that non-subtitle files don't get default language."""
    # Non-subtitle files without language should return None
    assert extract_language_from_companion_filename("poster.jpg", filepath="poster.jpg") is None
    assert extract_language_from_companion_filename("fanart.png", filepath="fanart.png") is None
    assert extract_language_from_companion_filename("movie.nfo", filepath="movie.nfo") is None
    assert extract_language_from_companion_filename("banner.jpeg", filepath="banner.jpeg") is None


def test_get_language_code_from_name():
    """Test language code conversion."""
    # Standard mappings
    assert get_language_code_from_name("english") == "en_us"
    assert get_language_code_from_name("spanish") == "es_es"
    assert get_language_code_from_name("french") == "fr_fr"
    assert get_language_code_from_name("german") == "de_de"
    assert get_language_code_from_name("italian") == "it_it"
    assert get_language_code_from_name("portuguese") == "pt_pt"
    assert get_language_code_from_name("russian") == "ru_ru"
    assert get_language_code_from_name("japanese") == "ja_jp"
    assert get_language_code_from_name("korean") == "ko_kr"
    assert get_language_code_from_name("chinese") == "zh_cn"
    assert get_language_code_from_name("arabic") == "ar_sa"
    assert get_language_code_from_name("hindi") == "hi_in"
    assert get_language_code_from_name("bulgarian") == "bg_bg"

    # Abbreviations
    assert get_language_code_from_name("eng") == "en_us"
    assert get_language_code_from_name("esp") == "es_es"
    assert get_language_code_from_name("fra") == "fr_fr"

    # Short codes
    assert get_language_code_from_name("en") == "en_us"
    assert get_language_code_from_name("es") == "es_es"
    assert get_language_code_from_name("fr") == "fr_fr"

    # Case insensitive
    assert get_language_code_from_name("ENGLISH") == "en_us"
    assert get_language_code_from_name("Spanish") == "es_es"
    assert get_language_code_from_name("FRENCH") == "fr_fr"

    # Unknown language returns as-is (lowercase)
    assert get_language_code_from_name("unknown") == "unknown"
    assert get_language_code_from_name("xyz") == "xyz"


def test_complete_workflow():
    """Test complete workflow: filename -> language name -> language code."""
    # English subtitle without explicit language
    filename = "movie.srt"
    lang_name = extract_language_from_companion_filename(filename, filepath=filename)
    assert lang_name == "english"
    lang_code = get_language_code_from_name(lang_name)
    assert lang_code == "en_us"

    # Spanish subtitle with explicit language
    filename = "Spanish.ass"
    lang_name = extract_language_from_companion_filename(filename, filepath=filename)
    assert lang_name == "spanish"
    lang_code = get_language_code_from_name(lang_name)
    assert lang_code == "es_es"

    # Non-subtitle image file
    filename = "poster.jpg"
    lang_name = extract_language_from_companion_filename(filename, filepath=filename)
    assert lang_name is None


def test_destination_filename_generation():
    """Test that destination filenames are correctly generated."""
    # Simulate the destination filename generation logic from main_window.py
    primary_stem = "Movie.Title.2024.1080p"

    # Subtitle with language
    companion_file = "Spanish.srt"
    lang_name = extract_language_from_companion_filename(companion_file, filepath=companion_file)
    lang_code = get_language_code_from_name(lang_name) if lang_name else ""
    assert lang_code == "es_es"

    companion_ext = ".srt"
    dest_name = f"{primary_stem}.{lang_code}{companion_ext}"
    assert dest_name == "Movie.Title.2024.1080p.es_es.srt"

    # Subtitle without language (defaults to en_us)
    companion_file = "subtitle.srt"
    lang_name = extract_language_from_companion_filename(companion_file, filepath=companion_file)
    lang_code = get_language_code_from_name(lang_name) if lang_name else ""
    assert lang_code == "en_us"

    dest_name = f"{primary_stem}.{lang_code}{companion_ext}"
    assert dest_name == "Movie.Title.2024.1080p.en_us.srt"

    # Non-subtitle without language
    companion_file = "poster.jpg"
    lang_name = extract_language_from_companion_filename(companion_file, filepath=companion_file)
    lang_code = get_language_code_from_name(lang_name) if lang_name else ""
    assert lang_code == ""

    companion_ext = ".jpg"
    if lang_code:
        dest_name = f"{primary_stem}.{lang_code}{companion_ext}"
    else:
        dest_name = f"{primary_stem}{companion_ext}"
    assert dest_name == "Movie.Title.2024.1080p.jpg"
