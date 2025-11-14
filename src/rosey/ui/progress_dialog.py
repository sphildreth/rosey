"""Progress dialog with cancel support."""

import contextlib

from PySide6.QtCore import Signal
from PySide6.QtWidgets import (
    QDialog,
    QLabel,
    QProgressBar,
    QPushButton,
    QTextEdit,
    QVBoxLayout,
    QWidget,
)


class ProgressDialog(QDialog):
    """Dialog showing progress with cancel button."""

    cancel_requested = Signal()
    close_and_clear_requested = Signal()

    def __init__(
        self, title: str = "Processing", parent: QWidget | None = None, dry_run: bool = False
    ) -> None:
        super().__init__(parent)
        self.setWindowTitle(title)
        self.setModal(True)
        self.setMinimumWidth(600)
        self.setMinimumHeight(400)
        self._cancelled = False
        self.dry_run = dry_run
        self._clear_on_close = False
        self._total_files = 0
        self._total_size_mb = 0.0
        self._total_time_sec = 0.0
        self.init_ui()

    def init_ui(self) -> None:
        """Initialize the UI."""
        layout = QVBoxLayout(self)

        # Dry run warning banner (if applicable)
        if self.dry_run:
            warning_label = QLabel("⚠️ DRY RUN MODE - No files will be moved ⚠️")
            warning_label.setStyleSheet(
                "QLabel { "
                "background-color: #ff9800; "
                "color: white; "
                "font-weight: bold; "
                "font-size: 14pt; "
                "padding: 10px; "
                "border-radius: 5px; "
                "}"
            )
            warning_label.setWordWrap(True)
            layout.addWidget(warning_label)

        # Status label
        self.status_label = QLabel("Starting...")
        layout.addWidget(self.status_label)

        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 100)
        self.progress_bar.setValue(0)
        layout.addWidget(self.progress_bar)

        # Statistics label
        self.stats_label = QLabel("Files: 0 | Size: 0.0 MB | Avg Time: 0.0s")
        self.stats_label.setStyleSheet("QLabel { color: #666; font-size: 9pt; }")
        layout.addWidget(self.stats_label)

        # Details log
        self.details_text = QTextEdit()
        self.details_text.setReadOnly(True)
        self.details_text.setMaximumHeight(150)
        layout.addWidget(self.details_text)

        # Cancel button
        self.cancel_button = QPushButton("Cancel")
        self.cancel_button.clicked.connect(self.on_cancel)
        layout.addWidget(self.cancel_button)

    def set_status(self, message: str) -> None:
        """Update status label."""
        self.status_label.setText(message)

    def set_progress(self, value: int, maximum: int = 100) -> None:
        """Update progress bar."""
        self.progress_bar.setMaximum(maximum)
        self.progress_bar.setValue(value)

    def append_detail(self, message: str) -> None:
        """Append message to details log."""
        self.details_text.append(message)

    def add_file_stats(self, filename: str, size_mb: float, time_sec: float) -> None:
        """Add statistics for a processed file."""
        self._total_files += 1
        self._total_size_mb += size_mb
        self._total_time_sec += time_sec

        # Update statistics label
        avg_time = self._total_time_sec / self._total_files if self._total_files > 0 else 0.0
        self.stats_label.setText(
            f"Files: {self._total_files} | "
            f"Total Size: {self._total_size_mb:.1f} MB | "
            f"Avg Time: {avg_time:.3f}s per file"
        )

        # Append to details log (with truncated filename if too long)
        display_name = filename if len(filename) <= 50 else f"...{filename[-47:]}"
        self.details_text.append(f"✓ {display_name} ({size_mb:.1f} MB, {time_sec:.3f}s)")

    def on_cancel(self) -> None:
        """Handle cancel button click."""
        self._cancelled = True
        self.cancel_button.setEnabled(False)
        self.cancel_button.setText("Cancelling...")
        self.status_label.setText("Cancelling operation...")
        self.cancel_requested.emit()

    def is_cancelled(self) -> bool:
        """Check if cancel was requested."""
        return self._cancelled

    def set_complete(self, success: bool = True, allow_clear: bool = False) -> None:
        """Mark operation as complete.

        Args:
            success: Whether operation completed successfully
            allow_clear: If True and success, show "Close and Clear" button option
        """
        # Ensure progress bar shows 100%
        self.progress_bar.setValue(self.progress_bar.maximum())

        # Reset cancellation state so button behaves correctly
        self._cancelled = False

        # Add final summary to details
        if success and self._total_files > 0:
            avg_time = self._total_time_sec / self._total_files
            self.details_text.append("")  # Blank line
            self.details_text.append("=" * 60)
            self.details_text.append(
                f"Summary: {self._total_files} files processed, "
                f"{self._total_size_mb:.1f} MB total, "
                f"{self._total_time_sec:.2f}s total time"
            )
            self.details_text.append(
                f"Average: {avg_time:.3f}s per file, "
                f"{self._total_size_mb / self._total_files:.2f} MB per file"
            )
            if self._total_time_sec > 0:
                throughput = self._total_size_mb / self._total_time_sec
                self.details_text.append(f"Throughput: {throughput:.2f} MB/s")
            self.details_text.append("=" * 60)

        if success:
            self.status_label.setText("✓ Complete!")
            self.status_label.setStyleSheet(
                "QLabel { color: green; font-weight: bold; font-size: 12pt; }"
            )

            # Disconnect all existing signals to prevent duplicate connections
            with contextlib.suppress(RuntimeError):
                self.cancel_button.clicked.disconnect()

            # Always enable the button and reset its state first
            self.cancel_button.setEnabled(True)

            if allow_clear:
                # Add "Close and Clear" button for successful operations
                self.cancel_button.setText("Close and Clear")
                self.cancel_button.setStyleSheet(
                    "QPushButton { "
                    "background-color: #4caf50; "
                    "color: white; "
                    "font-weight: bold; "
                    "padding: 8px 16px; "
                    "}"
                    "QPushButton:hover { "
                    "background-color: #45a049; "
                    "}"
                )
                self.cancel_button.clicked.connect(self.on_close_and_clear)
            else:
                self.cancel_button.setText("Close")
                self.cancel_button.setStyleSheet("")  # Reset to default style
                self.cancel_button.clicked.connect(self.accept)
        else:
            self.status_label.setText("✗ Failed - see details")
            self.status_label.setStyleSheet(
                "QLabel { color: red; font-weight: bold; font-size: 12pt; }"
            )
            self.cancel_button.setText("Close")
            self.cancel_button.setEnabled(True)
            self.cancel_button.setStyleSheet("")  # Reset to default style

            # Disconnect all existing signals
            with contextlib.suppress(RuntimeError):
                self.cancel_button.clicked.disconnect()

            self.cancel_button.clicked.connect(self.reject)

    def on_close_and_clear(self) -> None:
        """Handle Close and Clear button click."""
        self._clear_on_close = True
        self.close_and_clear_requested.emit()
        self.accept()
