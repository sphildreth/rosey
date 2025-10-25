"""Progress dialog with cancel support."""

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

    def __init__(self, title: str = "Processing", parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setWindowTitle(title)
        self.setModal(True)
        self.setMinimumWidth(500)
        self.setMinimumHeight(300)
        self._cancelled = False
        self.init_ui()

    def init_ui(self) -> None:
        """Initialize the UI."""
        layout = QVBoxLayout(self)

        # Status label
        self.status_label = QLabel("Starting...")
        layout.addWidget(self.status_label)

        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 100)
        self.progress_bar.setValue(0)
        layout.addWidget(self.progress_bar)

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

    def set_complete(self, success: bool = True) -> None:
        """Mark operation as complete."""
        if success:
            self.status_label.setText("Complete!")
            self.cancel_button.setText("Close")
            self.cancel_button.setEnabled(True)
            self.cancel_button.clicked.disconnect()
            self.cancel_button.clicked.connect(self.accept)
        else:
            self.status_label.setText("Failed - see details")
            self.cancel_button.setText("Close")
            self.cancel_button.setEnabled(True)
            self.cancel_button.clicked.disconnect()
            self.cancel_button.clicked.connect(self.reject)
