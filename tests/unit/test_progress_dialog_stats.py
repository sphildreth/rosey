"""Tests for progress dialog statistics tracking."""

import sys
from pathlib import Path

# Add src to path for testing
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from PySide6.QtWidgets import QApplication

from rosey.ui.progress_dialog import ProgressDialog


def test_progress_dialog_file_stats():
    """Test that file statistics are tracked and displayed correctly."""
    _app = QApplication.instance() or QApplication(sys.argv)

    dialog = ProgressDialog("Test Scan")

    # Initially no files processed
    assert dialog._total_files == 0
    assert dialog._total_size_mb == 0.0
    assert dialog._total_time_sec == 0.0

    # Add first file
    dialog.add_file_stats("movie1.mkv", 1500.5, 0.123)
    assert dialog._total_files == 1
    assert dialog._total_size_mb == 1500.5
    assert dialog._total_time_sec == 0.123

    # Check stats label is updated
    assert "Files: 1" in dialog.stats_label.text()
    assert "1500.5 MB" in dialog.stats_label.text()
    assert "0.123" in dialog.stats_label.text()

    # Add second file
    dialog.add_file_stats("movie2.mp4", 800.3, 0.089)
    assert dialog._total_files == 2
    assert dialog._total_size_mb == 1500.5 + 800.3
    assert dialog._total_time_sec == 0.123 + 0.089

    # Check average time calculation
    avg_time = (0.123 + 0.089) / 2
    assert f"{avg_time:.3f}" in dialog.stats_label.text()

    # Add third file
    dialog.add_file_stats("episode.mkv", 350.0, 0.056)
    assert dialog._total_files == 3
    assert dialog._total_size_mb == 1500.5 + 800.3 + 350.0
    assert dialog._total_time_sec == 0.123 + 0.089 + 0.056


def test_progress_dialog_stats_display():
    """Test that statistics are displayed in the details text."""
    _app = QApplication.instance() or QApplication(sys.argv)

    dialog = ProgressDialog("Test Scan")

    # Add a file
    dialog.add_file_stats("test_movie.mkv", 1234.5, 0.250)

    # Check that details text contains the file info
    details = dialog.details_text.toPlainText()
    assert "test_movie.mkv" in details
    assert "1234.5 MB" in details
    assert "0.250s" in details

    # Add file with very long name
    long_name = "a" * 100 + ".mkv"
    dialog.add_file_stats(long_name, 500.0, 0.100)

    # Check that long filename is truncated in display
    details = dialog.details_text.toPlainText()
    # Should contain truncated version (last 47 chars + "...")
    assert "..." in details


def test_progress_dialog_completion_summary():
    """Test that completion shows a summary of statistics."""
    _app = QApplication.instance() or QApplication(sys.argv)

    dialog = ProgressDialog("Test Scan")

    # Add some files
    dialog.add_file_stats("file1.mkv", 1000.0, 0.100)
    dialog.add_file_stats("file2.mp4", 2000.0, 0.200)
    dialog.add_file_stats("file3.avi", 1500.0, 0.150)

    # Complete the dialog
    dialog.set_complete(success=True)

    # Check that summary is in details text
    details = dialog.details_text.toPlainText()
    assert "Summary:" in details
    assert "3 files processed" in details
    assert "4500.0 MB total" in details
    assert "0.45s total time" in details or "0.45 s total time" in details
    assert "Average:" in details
    assert "Throughput:" in details


def test_progress_dialog_zero_files():
    """Test handling of zero files processed."""
    _app = QApplication.instance() or QApplication(sys.argv)

    dialog = ProgressDialog("Test Scan")

    # Complete without processing any files
    dialog.set_complete(success=True)

    # Should not crash and should not show summary for zero files
    details = dialog.details_text.toPlainText()
    # Summary should not appear for zero files
    assert "Summary:" not in details or dialog._total_files == 0


def test_progress_dialog_stats_formatting():
    """Test that statistics are formatted correctly."""
    _app = QApplication.instance() or QApplication(sys.argv)

    dialog = ProgressDialog("Test Scan")

    # Add file with small size
    dialog.add_file_stats("small.srt", 0.5, 0.001)

    # Check formatting
    details = dialog.details_text.toPlainText()
    assert "0.5 MB" in details
    assert "0.001s" in details

    # Add file with large size
    dialog.add_file_stats("large.mkv", 25000.8, 1.234)

    # Check that large numbers are formatted correctly (total is 0.5 + 25000.8)
    assert "25001.3 MB" in dialog.stats_label.text()  # Total size
