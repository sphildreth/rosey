"""Tests for companion file discovery."""


import pytest

from rosey.identifier.identifier import Identifier


@pytest.fixture
def temp_movie_dir(tmp_path):
    """Create a temporary directory for movie tests."""
    movie_dir = tmp_path / "test_movie"
    movie_dir.mkdir()
    return movie_dir


def test_companion_files_in_subs_folder(temp_movie_dir):
    """Test that SRT files in 'Subs' folder are discovered."""
    # Create movie file
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    # Create Subs folder with subtitle
    subs_dir = temp_movie_dir / "Subs"
    subs_dir.mkdir()
    (subs_dir / "en.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 1
    assert result.item.sidecars[0].endswith("Subs/en.srt")


def test_companion_files_case_insensitive(temp_movie_dir):
    """Test that subtitle folders are matched case-insensitively."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    # Test lowercase 'subs'
    subs_dir = temp_movie_dir / "subs"
    subs_dir.mkdir()
    (subs_dir / "en.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 1
    assert result.item.sidecars[0].endswith("subs/en.srt")


def test_companion_files_subtitles_folder(temp_movie_dir):
    """Test that 'Subtitles' folder is recognized."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    # Create Subtitles folder
    subs_dir = temp_movie_dir / "Subtitles"
    subs_dir.mkdir()
    (subs_dir / "en.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 1
    assert result.item.sidecars[0].endswith("Subtitles/en.srt")


def test_companion_files_nested_in_subfolder(temp_movie_dir):
    """Test that SRT files nested in subdirectories are discovered."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    # Create nested structure: Subs/English/en.srt
    subs_dir = temp_movie_dir / "Subs" / "English"
    subs_dir.mkdir(parents=True)
    (subs_dir / "en.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 1
    assert result.item.sidecars[0].endswith("Subs/English/en.srt")


def test_companion_files_multiple_formats(temp_movie_dir):
    """Test that multiple subtitle formats are discovered."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    subs_dir = temp_movie_dir / "Subs"
    subs_dir.mkdir()
    (subs_dir / "en.srt").touch()
    (subs_dir / "en.ass").touch()
    (subs_dir / "en.vtt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 3


def test_companion_files_deeply_nested(temp_movie_dir):
    """Test that deeply nested subtitle files are found."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    # Create deeply nested structure
    nested_dir = temp_movie_dir / "Subs" / "English" / "Forced"
    nested_dir.mkdir(parents=True)
    (nested_dir / "en.forced.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 1
    assert result.item.sidecars[0].endswith("en.forced.srt")


def test_companion_files_same_directory(temp_movie_dir):
    """Test that subtitle files in the same directory as movie are found."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    (temp_movie_dir / "Movie (2023).srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 1
    assert result.item.sidecars[0].endswith("Movie (2023).srt")


def test_companion_files_mixed_locations(temp_movie_dir):
    """Test companion files from both same directory and Subs folder."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    # Subtitle in same directory
    (temp_movie_dir / "Movie (2023).srt").touch()

    # Image in same directory
    (temp_movie_dir / "poster.jpg").touch()

    # Subtitle in Subs folder
    subs_dir = temp_movie_dir / "Subs"
    subs_dir.mkdir()
    (subs_dir / "en.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    # Should find 3 companions: 2 subtitles + 1 image
    assert len(result.item.sidecars) == 3


def test_companion_files_sub_folder_variant(temp_movie_dir):
    """Test that 'Sub' (singular) folder is recognized."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    sub_dir = temp_movie_dir / "Sub"
    sub_dir.mkdir()
    (sub_dir / "en.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 1
    assert result.item.sidecars[0].endswith("Sub/en.srt")


def test_companion_files_subtitle_folder_variant(temp_movie_dir):
    """Test that 'Subtitle' (singular) folder is recognized."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    subtitle_dir = temp_movie_dir / "Subtitle"
    subtitle_dir.mkdir()
    (subtitle_dir / "en.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    assert len(result.item.sidecars) == 1
    assert result.item.sidecars[0].endswith("Subtitle/en.srt")


def test_companion_files_ignores_other_folders(temp_movie_dir):
    """Test that non-subtitle folders are not scanned."""
    movie_file = temp_movie_dir / "Movie (2023).mkv"
    movie_file.touch()

    # Create a folder that should NOT be scanned
    other_dir = temp_movie_dir / "Extras"
    other_dir.mkdir()
    (other_dir / "should_not_find.srt").touch()

    identifier = Identifier()
    result = identifier.identify(str(movie_file))

    # Should find no companions (the .srt is in a non-subtitle folder)
    assert len(result.item.sidecars) == 0
