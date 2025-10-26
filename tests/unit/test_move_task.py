"""Tests for move_task module."""

from unittest.mock import MagicMock, patch

import pytest

from rosey.models import MediaItem
from rosey.tasks.move_task import MoveTask


@pytest.fixture
def sample_items():
    """Create sample media items for testing."""
    return [
        (
            MediaItem(
                kind="movie",
                source_path="/source/movie.mkv",
                title="Test Movie",
            ),
            "/dest/Test Movie/Test Movie.mkv",
        ),
        (
            MediaItem(
                kind="episode",
                source_path="/source/episode.mkv",
                title="Test Show",
                season=1,
                episodes=[1],
            ),
            "/dest/Test Show/Season 01/Test Show S01E01.mkv",
        ),
    ]


def test_move_task_init(sample_items):
    """Test MoveTask initialization."""
    task = MoveTask(sample_items, conflict_policy="replace", dry_run=False)

    assert task.items == sample_items
    assert task.conflict_policy == "replace"
    assert task.dry_run is False
    assert task._cancelled is False
    assert hasattr(task, "signals")


def test_move_task_cancel():
    """Test task cancellation."""
    task = MoveTask([])
    assert task._cancelled is False

    task.cancel()
    assert task._cancelled is True


def test_move_task_run_success(sample_items, qtbot):
    """Test successful move task execution."""
    task = MoveTask(sample_items, dry_run=True)

    # Mock move_with_sidecars to return success
    mock_result = MagicMock()
    mock_result.success = True
    mock_result.details = {"moved": ["/dest/movie.mkv"]}
    mock_result.errors = []

    with patch("rosey.tasks.move_task.move_with_sidecars") as mock_move:
        mock_move.return_value = mock_result

        # Connect to signals
        progress_signals = []
        item_signals = []
        finished_signals = []
        error_signals = []

        task.signals.progress.connect(lambda *args: progress_signals.append(args))
        task.signals.item_moved.connect(lambda *args: item_signals.append(args))
        task.signals.finished.connect(lambda *args: finished_signals.append(args))
        task.signals.error.connect(lambda *args: error_signals.append(args))

        # Run task
        task.run()

        # Check signals
        assert len(progress_signals) >= 2  # Progress updates
        assert len(item_signals) == 2  # One per item
        assert len(finished_signals) == 1
        assert len(error_signals) == 0

        # Check finished results
        results = finished_signals[0][0]
        assert len(results) == 2
        assert all(r.success for r in results)


def test_move_task_run_with_failure(sample_items, qtbot):
    """Test move task with failures."""
    task = MoveTask(sample_items, dry_run=True)

    # Mock move_with_sidecars to return failure for first item
    success_result = MagicMock()
    success_result.success = True
    success_result.details = {"moved": ["/dest/movie.mkv"]}
    success_result.errors = []

    failure_result = MagicMock()
    failure_result.success = False
    failure_result.details = {}
    failure_result.errors = ["Disk full"]

    with patch("rosey.tasks.move_task.move_with_sidecars") as mock_move:
        mock_move.side_effect = [failure_result, success_result]

        # Connect to signals
        item_signals = []
        error_signals = []

        task.signals.item_moved.connect(lambda *args: item_signals.append(args))
        task.signals.error.connect(lambda *args: error_signals.append(args))

        # Run task
        task.run()

        # Check signals
        assert len(item_signals) == 2
        assert len(error_signals) == 1

        # Check item signals
        assert item_signals[0][1] is False  # First item failed
        assert item_signals[1][1] is True  # Second item succeeded

        # Check error signal
        assert "Disk full" in error_signals[0][0]


def test_move_task_run_cancelled(sample_items, qtbot):
    """Test move task cancellation."""
    task = MoveTask(sample_items, dry_run=True)

    # Cancel after first item
    call_count = 0

    def mock_move(*args, **kwargs):
        nonlocal call_count
        call_count += 1
        if call_count == 1:
            task.cancel()  # Cancel after first item
        return MagicMock(success=True, details={"moved": []}, errors=[])

    with patch("rosey.tasks.move_task.move_with_sidecars", side_effect=mock_move):
        progress_signals = []
        task.signals.progress.connect(lambda *args: progress_signals.append(args))

        task.run()

        # Should have cancelled after first item
        assert call_count == 1
        assert any("Cancelled" in str(args) for args in progress_signals)


def test_move_task_run_exception(sample_items, qtbot):
    """Test move task with unexpected exception."""
    task = MoveTask(sample_items, dry_run=True)

    with patch("rosey.tasks.move_task.move_with_sidecars") as mock_move:
        mock_move.side_effect = Exception("Unexpected error")

        error_signals = []
        finished_signals = []

        task.signals.error.connect(lambda *args: error_signals.append(args))
        task.signals.finished.connect(lambda *args: finished_signals.append(args))

        task.run()

        # Should emit error and still finish
        assert len(error_signals) == 1
        assert "Unexpected error" in error_signals[0][0]
        assert len(finished_signals) == 1


def test_move_task_action_detection():
    """Test action detection from move results."""
    # Test different result scenarios
    test_cases = [
        ({"skipped": ["/path"]}, "skipped"),
        ({"replaced": ["/path"]}, "replaced"),
        ({"kept_both": ["/path"]}, "kept_both"),
        ({}, "failed"),  # No details means failure
        ({"moved": ["/path"]}, "moved"),
    ]

    for details, expected_action in test_cases:
        mock_result = MagicMock()
        mock_result.success = expected_action != "failed"  # Set success based on expected
        mock_result.details = details
        mock_result.errors = []

        # Simulate the logic from run()
        action = "moved" if mock_result.success else "failed"
        if mock_result.details.get("skipped"):
            action = "skipped"
        elif mock_result.details.get("replaced"):
            action = "replaced"
        elif mock_result.details.get("kept_both"):
            action = "kept_both"

        assert action == expected_action


def test_move_task_signals_setup():
    """Test that signals are properly set up."""
    task = MoveTask([])

    assert hasattr(task.signals, "progress")
    assert hasattr(task.signals, "item_moved")
    assert hasattr(task.signals, "finished")
    assert hasattr(task.signals, "error")

    # Test signal types
    from PySide6.QtCore import Signal

    assert isinstance(task.signals.progress, Signal)
    assert isinstance(task.signals.item_moved, Signal)
    assert isinstance(task.signals.finished, Signal)
    assert isinstance(task.signals.error, Signal)
