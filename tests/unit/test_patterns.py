"""Tests for identifier patterns."""


from rosey.identifier.patterns import (
    clean_title,
    extract_date,
    extract_episode_info,
    extract_part,
    extract_season_from_folder,
    extract_year,
)


class TestEpisodePatterns:
    """Test episode pattern extraction."""

    def test_sxxeyy_format(self):
        """Test S01E02 format."""
        result = extract_episode_info("Show.S01E05.mkv")
        assert result is not None
        assert result.season == 1
        assert result.episodes == [5]

    def test_sxxeyy_range(self):
        """Test S01E02-E03 format."""
        result = extract_episode_info("Show.S02E10-E11.mkv")
        assert result is not None
        assert result.season == 2
        assert result.episodes == [10, 11]

    def test_xformat(self):
        """Test 1x02 format."""
        result = extract_episode_info("Show.1x05.mkv")
        assert result is not None
        assert result.season == 1
        assert result.episodes == [5]

    def test_case_insensitive(self):
        """Test case insensitivity."""
        result1 = extract_episode_info("show.s01e05.mkv")
        result2 = extract_episode_info("SHOW.S01E05.MKV")

        assert result1 is not None
        assert result2 is not None
        assert result1.season == result2.season
        assert result1.episodes == result2.episodes

    def test_no_episode_pattern(self):
        """Test filename with no episode pattern."""
        result = extract_episode_info("Just a Movie (2020).mkv")
        assert result is None

    def test_multi_digit_episode(self):
        """Test multi-digit episode numbers."""
        result = extract_episode_info("Show.S01E123.mkv")
        assert result is not None
        assert result.season == 1
        assert result.episodes == [123]


class TestDatePatterns:
    """Test date pattern extraction."""

    def test_date_with_dashes(self):
        """Test YYYY-MM-DD format."""
        result = extract_date("Show.2020-03-15.mkv")
        assert result is not None
        assert result.date == "2020-03-15"

    def test_date_with_dots(self):
        """Test YYYY.MM.DD format."""
        result = extract_date("Show.2021.12.25.mkv")
        assert result is not None
        assert result.date == "2021-12-25"

    def test_no_date_pattern(self):
        """Test filename with no date."""
        result = extract_date("Movie (2020).mkv")
        assert result is None


class TestYearPatterns:
    """Test year extraction."""

    def test_year_in_parentheses(self):
        """Test (1999) format."""
        result = extract_year("The Matrix (1999).mkv")
        assert result == 1999

    def test_year_standalone(self):
        """Test standalone year."""
        result = extract_year("Movie 2020.mkv")
        assert result == 2020

    def test_no_year(self):
        """Test no year present."""
        result = extract_year("Movie Title.mkv")
        assert result is None

    def test_multiple_years(self):
        """Test multiple years (should pick first)."""
        result = extract_year("Movie (1999) vs (2020).mkv")
        assert result in [1999, 2020]


class TestPartPatterns:
    """Test part number extraction."""

    def test_part_with_space(self):
        """Test 'Part 1' format."""
        result = extract_part("Movie Part 1.mkv")
        assert result == 1

    def test_part_abbreviated(self):
        """Test 'pt1' format."""
        result = extract_part("Movie pt2.mkv")
        assert result == 2

    def test_part_with_dot(self):
        """Test 'Part.3' format."""
        result = extract_part("Movie.Part.3.mkv")
        assert result == 3

    def test_no_part(self):
        """Test no part indicator."""
        result = extract_part("Movie.mkv")
        assert result is None


class TestSeasonFolderPatterns:
    """Test season folder pattern extraction."""

    def test_season_standard(self):
        """Test 'Season 01' format."""
        result = extract_season_from_folder("Season 01")
        assert result == 1

    def test_season_no_zero_pad(self):
        """Test 'Season 1' format."""
        result = extract_season_from_folder("Season 1")
        assert result == 1

    def test_season_case_insensitive(self):
        """Test case insensitivity."""
        result = extract_season_from_folder("season 02")
        assert result == 2

    def test_no_season_pattern(self):
        """Test folder with no season pattern."""
        result = extract_season_from_folder("Random Folder")
        assert result is None


class TestTitleCleaning:
    """Test title cleaning."""

    def test_clean_basic(self):
        """Test basic cleaning."""
        result = clean_title("The.Matrix.1999.1080p.x264")
        assert "Matrix" in result
        assert "1999" not in result
        assert "1080p" not in result
        assert "x264" not in result

    def test_clean_episode_pattern(self):
        """Test removing episode patterns."""
        result = clean_title("Show.S01E05.Episode.Title")
        assert "S01E05" not in result
        assert "Show" in result

    def test_clean_separators(self):
        """Test normalizing separators."""
        result = clean_title("Movie_Name-With.Separators")
        assert "_" not in result
        assert "-" not in result
        assert "." not in result
        assert " " in result

    def test_clean_multiple_spaces(self):
        """Test collapsing multiple spaces."""
        result = clean_title("Movie   Name   Here")
        assert "  " not in result
        assert result == "Movie Name Here"
