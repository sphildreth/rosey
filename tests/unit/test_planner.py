"""Tests for planner module."""

from rosey.models import MediaItem
from rosey.planner import Planner, plan_path, sanitize_name


class TestSanitizeName:
    """Test filename sanitization."""

    def test_sanitize_basic(self):
        """Test basic sanitization."""
        result = sanitize_name("Normal Movie Name")
        assert result == "Normal Movie Name"

    def test_sanitize_invalid_chars(self):
        """Test removing invalid characters."""
        result = sanitize_name('Movie<>:"/\\|?*Name')
        assert "<" not in result
        assert ">" not in result
        assert ":" not in result
        assert '"' not in result
        assert "/" not in result
        assert "\\" not in result
        assert "|" not in result
        assert "?" not in result
        assert "*" not in result

    def test_sanitize_multiple_spaces(self):
        """Test collapsing multiple spaces."""
        result = sanitize_name("Movie   Name   Here")
        assert "  " not in result
        assert result == "Movie Name Here"

    def test_sanitize_leading_trailing(self):
        """Test removing leading/trailing spaces and dots."""
        result = sanitize_name("  Movie Name.  ")
        assert not result.startswith(" ")
        assert not result.endswith(" ")
        assert not result.endswith(".")

    def test_sanitize_reserved_names(self):
        """Test Windows reserved names."""
        reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"]

        for name in reserved:
            result = sanitize_name(name)
            # Should be modified
            assert result != name

    def test_sanitize_empty(self):
        """Test empty string."""
        result = sanitize_name("")
        assert result == "unknown"


class TestPlannerMovies:
    """Test movie path planning."""

    def test_movie_basic(self):
        """Test basic movie path."""
        planner = Planner(movies_root="/movies")

        item = MediaItem(
            kind="movie",
            source_path="/source/file.mkv",
            title="The Matrix",
            year=1999,
        )

        dest = planner.plan_destination(item)

        assert "/movies" in dest
        assert "The Matrix" in dest
        assert "(1999)" in dest
        assert dest.endswith(".mkv")

    def test_movie_without_year(self):
        """Test movie without year."""
        planner = Planner(movies_root="/movies")

        item = MediaItem(
            kind="movie",
            source_path="/source/Inception.mp4",
            title="Inception",
        )

        dest = planner.plan_destination(item)

        assert "/movies" in dest
        assert "Inception" in dest
        assert dest.endswith(".mp4")

    def test_movie_multipart(self):
        """Test multipart movie."""
        planner = Planner(movies_root="/movies")

        item = MediaItem(
            kind="movie",
            source_path="/source/file.mkv",
            title="Kill Bill",
            year=2003,
            part=1,
        )

        dest = planner.plan_destination(item)

        assert "Part 1" in dest

    def test_movie_no_root(self):
        """Test movie with no movies root set."""
        planner = Planner(movies_root="")

        item = MediaItem(
            kind="movie",
            source_path="/source/file.mkv",
            title="Movie",
        )

        dest = planner.plan_destination(item)

        # Should return original path
        assert dest == "/source/file.mkv"


class TestPlannerTVShows:
    """Test TV show path planning."""

    def test_episode_basic(self):
        """Test basic episode path."""
        planner = Planner(tv_root="/tv")

        item = MediaItem(
            kind="episode",
            source_path="/source/file.mkv",
            title="The Office",
            season=2,
            episodes=[1],
        )

        dest = planner.plan_destination(item)

        assert "/tv" in dest
        assert "The Office" in dest
        assert "Season 02" in dest
        assert "S02E01" in dest
        assert dest.endswith(".mkv")

    def test_episode_multi_episode(self):
        """Test multi-episode path."""
        planner = Planner(tv_root="/tv")

        item = MediaItem(
            kind="episode",
            source_path="/source/file.mkv",
            title="Show",
            season=1,
            episodes=[1, 2],
        )

        dest = planner.plan_destination(item)

        assert "S01E01-E02" in dest

    def test_episode_with_title(self):
        """Test episode with episode title."""
        planner = Planner(tv_root="/tv")

        item = MediaItem(
            kind="episode",
            source_path="/source/file.mkv",
            title="The Office",
            season=2,
            episodes=[1],
            nfo={"episode_title": "The Dundies"},
        )

        dest = planner.plan_destination(item)

        assert "The Dundies" in dest

    def test_episode_date_based(self):
        """Test date-based episode."""
        planner = Planner(tv_root="/tv")

        item = MediaItem(
            kind="episode",
            source_path="/source/file.mkv",
            title="Daily Show",
            date="2020-03-15",
        )

        dest = planner.plan_destination(item)

        assert "2020-03-15" in dest

    def test_episode_multipart(self):
        """Test multipart episode."""
        planner = Planner(tv_root="/tv")

        item = MediaItem(
            kind="episode",
            source_path="/source/file.mkv",
            title="Show",
            season=1,
            episodes=[1],
            part=1,
        )

        dest = planner.plan_destination(item)

        assert "Part 1" in dest

    def test_episode_specials(self):
        """Test specials (Season 0)."""
        planner = Planner(tv_root="/tv")

        item = MediaItem(
            kind="episode",
            source_path="/source/file.mkv",
            title="Show",
            season=0,
            episodes=[1],
        )

        dest = planner.plan_destination(item)

        assert "Season 00" in dest
        assert "S00E01" in dest

    def test_episode_no_root(self):
        """Test episode with no TV root set."""
        planner = Planner(tv_root="")

        item = MediaItem(
            kind="episode",
            source_path="/source/file.mkv",
            title="Show",
            season=1,
            episodes=[1],
        )

        dest = planner.plan_destination(item)

        # Should return original path
        assert dest == "/source/file.mkv"

    def test_episode_all_in_the_family_s6e9(self):
        """Test specific episode destination for 'All in the Family' S6E9."""
        planner = Planner(tv_root="/mnt/fileserver_storage/videos/tv")

        item = MediaItem(
            kind="episode",
            source_path="/mnt/fileserver_incoming/complete/All in the family (1971) [tmdbid-1922]/All in the family (Archie Bunker US TV Series) S6EP09 Grandpa blues (moviesbyrizzo).mp4",
            title="All in the family",
            year=1971,
            season=6,
            episodes=[9],
            nfo={"tmdbid": "1922", "episode_title": "Grandpa blues"},
        )

        dest = planner.plan_destination(item)

        expected = "/mnt/fileserver_storage/videos/tv/All in the Family (1971) [tmdbid-1922]/Season 06/All in the Family - S06E09 - Grandpa blues.mp4"
        assert dest == expected


class TestPlannerUnknown:
    """Test unknown media handling."""

    def test_unknown_returns_source(self):
        """Test that unknown media returns source path."""
        planner = Planner(movies_root="/movies", tv_root="/tv")

        item = MediaItem(
            kind="unknown",
            source_path="/source/mystery.mkv",
        )

        dest = planner.plan_destination(item)

        assert dest == "/source/mystery.mkv"


def test_plan_path_convenience():
    """Test convenience function."""
    item = MediaItem(
        kind="movie",
        source_path="/source/file.mkv",
        title="Movie",
        year=2020,
    )

    dest = plan_path(item, movies_root="/movies")

    assert "/movies" in dest
    assert "Movie" in dest
    assert "(2020)" in dest
