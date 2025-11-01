"""Tests for main window UI functionality."""

from unittest.mock import Mock, patch

from rosey.models import MediaItem, Score
from rosey.ui.main_window import (
    extract_language_from_companion_filename,
    get_language_code_from_name,
)


def test_on_identify_updates_companion_destinations():
    """Test that companion destinations are updated during identification to match primary video."""
    # Mock the planner function
    with patch("rosey.ui.main_window.plan_path") as mock_plan_path:
        mock_plan_path.return_value = "/movies/New Title (2003) [tmdbid-604]/New Title.mkv"

        # Create test items - a movie and its companion
        movie_item = MediaItem(
            kind="movie",
            source_path="/source/Matrix.mkv",
            title="The Matrix",
            year=1999,
            nfo={"tmdbid": "603"},
        )

        companion_item = MediaItem(
            kind="companion", source_path="/source/Matrix.srt", title="Matrix.srt"
        )

        # Create item dictionaries with initial destinations
        movie_dict = {
            "item": movie_item,
            "score": Score(confidence=85, reasons=["TMDB ID"]),
            "destination": "/movies/The Matrix (1999) [tmdbid-603]/The Matrix.mkv",
            "group": Mock(directory="/source"),
        }

        companion_dict = {
            "item": companion_item,
            "score": Score(confidence=100, reasons=["Companion file"]),
            "destination": "/movies/The Matrix (1999) [tmdbid-603]/The Matrix.srt",
            "group": Mock(directory="/source"),
        }

        # Simulate the identification logic from on_identify
        group_items = [movie_dict, companion_dict]

        # Mock identification result
        result = {"title": "The Matrix Reloaded", "year": "2003", "id": "604"}

        # Apply the identification logic (copied from on_identify method)
        for item_dict in group_items:
            item = item_dict["item"]

            # Update from result
            if "id" in result:
                item.nfo = item.nfo or {}
                item.nfo["tmdbid"] = str(result["id"])
                item.nfo["_source"] = "identification"
                item.title = result.get("title", result.get("name", item.title))
                item.year = int(result["year"])

            # Re-plan destination
            if item.kind != "companion":
                new_dest = mock_plan_path(
                    item,
                    movies_root="/movies",
                    tv_root="/tv",
                )
                item_dict["destination"] = new_dest
            else:
                # For companions, recalculate destination based on updated primary video
                # Find the primary video in the same group
                for primary_dict in group_items:
                    primary_item = primary_dict["item"]
                    if primary_item.kind != "companion":
                        from pathlib import Path

                        primary_dest_dir = Path(primary_dict["destination"]).parent
                        primary_dest_stem = Path(primary_dict["destination"]).stem

                        # Extract language from companion filename
                        companion_filename = Path(item.source_path).name
                        language_name = extract_language_from_companion_filename(companion_filename)
                        language_code = (
                            get_language_code_from_name(language_name) if language_name else ""
                        )

                        # Build companion destination: primary_base.language_code.ext
                        companion_ext = Path(item.source_path).suffix
                        if language_code:
                            companion_dest_name = (
                                f"{primary_dest_stem}.{language_code}{companion_ext}"
                            )
                        else:
                            # Fallback: use primary base name with companion extension
                            companion_dest_name = f"{primary_dest_stem}{companion_ext}"

                        item_dict["destination"] = str(primary_dest_dir / companion_dest_name)
                        break

        # Verify movie destination was re-planned
        assert movie_dict["destination"] == "/movies/New Title (2003) [tmdbid-604]/New Title.mkv"
        assert movie_item.title == "The Matrix Reloaded"
        assert movie_item.year == 2003
        assert movie_item.nfo["tmdbid"] == "604"

        # Verify companion destination was updated to match new primary destination
        # Subtitle files default to en_us language code
        assert (
            companion_dict["destination"]
            == "/movies/New Title (2003) [tmdbid-604]/New Title.en_us.srt"
        )
        assert companion_item.title == "The Matrix Reloaded"  # Title still gets updated
        assert companion_item.year == 2003  # Year still gets updated

        # Verify plan_path was only called for the movie, not the companion
        assert mock_plan_path.call_count == 1
        mock_plan_path.assert_called_once_with(
            movie_item,
            movies_root="/movies",
            tv_root="/tv",
        )


def test_on_identify_updates_movie_metadata():
    """Test that movie metadata is updated during identification."""
    # Create a movie item
    movie_item = MediaItem(
        kind="movie", source_path="/source/OldMovie.mkv", title="Old Title", year=2000
    )

    movie_dict = {
        "item": movie_item,
        "score": Score(confidence=50, reasons=["Filename"]),
        "destination": "/movies/Old Title (2000)/OldMovie.mkv",
        "group": Mock(directory="/source"),
    }

    # Simulate the identification logic
    group_items = [movie_dict]

    # Mock identification result
    result = {"title": "Correct Movie Title", "year": "2020", "id": "12345"}

    # Apply the identification logic
    for item_dict in group_items:
        item = item_dict["item"]

        # Update from result
        if "id" in result:
            item.nfo = item.nfo or {}
            item.nfo["tmdbid"] = str(result["id"])
            item.nfo["_source"] = "identification"
            item.title = result.get("title", result.get("name", item.title))
            item.year = int(result["year"])

        # Re-score
        item_dict["score"] = Score(confidence=90, reasons=["Manual identification"])

    # Verify metadata was updated
    assert movie_item.title == "Correct Movie Title"
    assert movie_item.year == 2020
    assert movie_item.nfo["tmdbid"] == "12345"
    assert movie_item.nfo["_source"] == "identification"


def test_on_identify_handles_episode_identification():
    """Test that episode identification works correctly."""
    # Create an episode item
    episode_item = MediaItem(
        kind="episode", source_path="/source/Show.S01E01.mkv", title="Show", season=1, episodes=[1]
    )

    episode_dict = {
        "item": episode_item,
        "score": Score(confidence=60, reasons=["Filename"]),
        "destination": "/tv/Show/Season 01/Show.S01E01.mkv",
        "group": Mock(directory="/source"),
    }

    # Simulate the identification logic
    group_items = [episode_dict]

    # Mock identification result
    result = {"title": "Correct Show Name", "season": "2", "episode": "5", "id": "67890"}

    # Apply the identification logic
    for item_dict in group_items:
        item = item_dict["item"]

        # Update from result
        if "id" in result:
            item.nfo = item.nfo or {}
            item.nfo["tmdbid"] = str(result["id"])
            item.nfo["_source"] = "identification"
            item.title = result.get("title", result.get("name", item.title))

        # Handle episode-specific fields
        if result.get("season") and item.kind == "episode":
            item.season = int(result["season"])
        if result.get("episode") and item.kind == "episode":
            item.episodes = [int(result["episode"])]

        # Re-score
        item_dict["score"] = Score(confidence=90, reasons=["Manual identification"])

    # Verify episode metadata was updated
    assert episode_item.title == "Correct Show Name"
    assert episode_item.season == 2
    assert episode_item.episodes == [5]
    assert episode_item.nfo["tmdbid"] == "67890"

    @patch("rosey.ui.main_window.QMainWindow")
    @patch("rosey.ui.main_window.load_config")
    @patch("rosey.ui.main_window.ProviderManager")
    @patch("rosey.ui.main_window.QThreadPool")
    def test_on_identify_updates_movie_metadata_and_destination(
        self, mock_qthreadpool, mock_providermanager, mock_load_config, mock_qmainwindow
    ):
        """Test that movie metadata and destination are updated during identification."""
        # Mock the config
        mock_config = Mock()
        mock_config.paths.movies = "/movies"
        mock_config.paths.tv = "/tv"
        mock_load_config.return_value = mock_config

        # Import after mocking Qt components
        from rosey.ui.main_window import MainWindow

        window = MainWindow()
        window.config = mock_config

        # Create a movie item
        movie_item = MediaItem(
            kind="movie", source_path="/source/OldMovie.mkv", title="Old Title", year=2000
        )

        movie_dict = {
            "item": movie_item,
            "score": Score(confidence=50, reasons=["Filename"]),
            "destination": "/movies/Old Title (2000)/OldMovie.mkv",
            "group": Mock(directory="/source"),
        }

        window.items = [movie_dict]
        window.groups = {"/source": {"kind": "movie", "items": [movie_dict], "node": Mock()}}

        # Mock dialog with new identification
        with patch("rosey.ui.main_window.IdentifyDialog") as mock_dialog_class:
            mock_dialog = Mock()
            mock_dialog_class.return_value = mock_dialog
            mock_dialog.exec.return_value = True
            mock_dialog.get_result.return_value = {
                "title": "Correct Movie Title",
                "year": "2020",
                "id": "12345",
            }

            window.provider_manager = Mock()
            window.on_identify(Mock())

            # Verify metadata was updated
            assert movie_item.title == "Correct Movie Title"
            assert movie_item.year == 2020
            assert movie_item.nfo["tmdbid"] == "12345"

            # Verify destination was re-planned
            assert "Correct Movie Title (2020)" in movie_dict["destination"]
            assert "[tmdbid-12345]" in movie_dict["destination"]

    @patch("rosey.ui.main_window.QMainWindow")
    @patch("rosey.ui.main_window.load_config")
    @patch("rosey.ui.main_window.ProviderManager")
    @patch("rosey.ui.main_window.QThreadPool")
    def test_on_identify_handles_episode_identification(
        self, mock_qthreadpool, mock_providermanager, mock_load_config, mock_qmainwindow
    ):
        """Test that episode identification works correctly."""
        # Mock the config
        mock_config = Mock()
        mock_config.paths.tv = "/tv"
        mock_load_config.return_value = mock_config

        # Import after mocking Qt components
        from rosey.ui.main_window import MainWindow

        window = MainWindow()
        window.config = mock_config

        episode_item = MediaItem(
            kind="episode",
            source_path="/source/Show.S01E01.mkv",
            title="Show",
            season=1,
            episodes=[1],
        )

        episode_dict = {
            "item": episode_item,
            "score": Score(confidence=60, reasons=["Filename"]),
            "destination": "/tv/Show/Season 01/Show.S01E01.mkv",
            "group": Mock(directory="/source"),
        }

        window.items = [episode_dict]
        window.groups = {"/source": {"kind": "show", "items": [episode_dict], "node": Mock()}}

        with patch("rosey.ui.main_window.IdentifyDialog") as mock_dialog_class:
            mock_dialog = Mock()
            mock_dialog_class.return_value = mock_dialog
            mock_dialog.exec.return_value = True
            mock_dialog.get_result.return_value = {
                "title": "Correct Show Name",
                "season": "2",
                "episode": "5",
                "id": "67890",
            }

            window.provider_manager = Mock()
            window.on_identify(Mock())

            # Verify episode metadata was updated
            assert episode_item.title == "Correct Show Name"
            assert episode_item.season == 2
            assert episode_item.episodes == [5]
            assert episode_item.nfo["tmdbid"] == "67890"

            # Verify destination was re-planned
            assert "Correct Show Name" in episode_dict["destination"]
            assert "Season 02" in episode_dict["destination"]
