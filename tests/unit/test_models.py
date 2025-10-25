"""Tests for data models."""

from rosey.models import (
    IdentificationResult,
    MediaItem,
    MovePlan,
    MoveResult,
    Score,
)


def test_media_item_movie() -> None:
    """Test creating a movie media item."""
    item = MediaItem(
        kind="movie",
        source_path="/path/to/The Matrix (1999).mkv",
        title="The Matrix",
        year=1999,
    )

    assert item.kind == "movie"
    assert item.title == "The Matrix"
    assert item.year == 1999
    assert item.season is None
    assert item.episodes is None


def test_media_item_episode() -> None:
    """Test creating an episode media item."""
    item = MediaItem(
        kind="episode",
        source_path="/path/to/The Office S02E01.mkv",
        title="The Office",
        season=2,
        episodes=[1],
    )

    assert item.kind == "episode"
    assert item.season == 2
    assert item.episodes == [1]


def test_media_item_multi_episode() -> None:
    """Test creating a multi-episode media item."""
    item = MediaItem(
        kind="episode",
        source_path="/path/to/Show S01E01-E02.mkv",
        title="Show",
        season=1,
        episodes=[1, 2],
    )

    assert item.episodes == [1, 2]


def test_identification_result() -> None:
    """Test identification result."""
    item = MediaItem(kind="movie", source_path="/path/to/file.mkv", title="Test Movie")

    result = IdentificationResult(
        item=item,
        reasons=["Matched filename pattern", "Found TMDB ID in NFO"],
    )

    assert len(result.reasons) == 2
    assert result.item.title == "Test Movie"
    assert result.errors == []


def test_score() -> None:
    """Test confidence score."""
    score = Score(
        confidence=85,
        reasons=["NFO with TMDB ID", "Season/episode pattern match"],
    )

    assert score.confidence == 85
    assert len(score.reasons) == 2


def test_move_plan() -> None:
    """Test move plan."""
    plan = MovePlan(
        destination_paths=["/target/Movie (2020).mkv"],
        dry_run=True,
    )

    assert plan.dry_run is True
    assert len(plan.destination_paths) == 1
    assert plan.preflight["free_space_ok"] is True


def test_move_result_success() -> None:
    """Test successful move result."""
    result = MoveResult(
        success=True,
        details={"moved": ["/src/file.mkv"], "skipped": []},
    )

    assert result.success is True
    assert len(result.details["moved"]) == 1
    assert result.rollback_performed is False


def test_move_result_with_rollback() -> None:
    """Test move result with rollback."""
    result = MoveResult(
        success=False,
        rollback_performed=True,
        errors=["Copy verification failed"],
    )

    assert result.success is False
    assert result.rollback_performed is True
    assert len(result.errors) == 1
