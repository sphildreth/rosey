"""Conflict resolution dialog."""

from PySide6.QtWidgets import (
    QDialog,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QRadioButton,
    QVBoxLayout,
    QWidget,
)


class ConflictDialog(QDialog):
    """Dialog to resolve file conflicts."""

    def __init__(self, source_path: str, dest_path: str, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.source_path = source_path
        self.dest_path = dest_path
        self.conflict_policy = "skip"
        self.abort_all = False
        self.init_ui()

    def init_ui(self) -> None:
        """Initialize the UI."""
        self.setWindowTitle("File Conflict")
        self.setModal(True)
        self.setMinimumWidth(500)

        layout = QVBoxLayout(self)

        # Message
        msg = QLabel(
            f"The destination file already exists:\n\n{self.dest_path}\n\n"
            f"Source: {self.source_path}\n\n"
            "How would you like to proceed?"
        )
        msg.setWordWrap(True)
        layout.addWidget(msg)

        # Radio buttons
        self.rb_skip = QRadioButton("Skip - Don't move this file")
        self.rb_skip.setChecked(True)
        layout.addWidget(self.rb_skip)

        self.rb_replace = QRadioButton("Replace - Overwrite the existing file")
        layout.addWidget(self.rb_replace)

        self.rb_keep_both = QRadioButton("Keep Both - Rename the new file with (1) suffix")
        layout.addWidget(self.rb_keep_both)

        # Buttons layout
        button_layout = QHBoxLayout()

        # Apply button
        self.apply_button = QPushButton("Apply")
        self.apply_button.clicked.connect(self.on_apply)
        button_layout.addWidget(self.apply_button)

        # Skip button
        self.skip_button = QPushButton("Skip This File")
        self.skip_button.clicked.connect(self.on_skip)
        button_layout.addWidget(self.skip_button)

        # Abort button (larger and more prominent)
        self.abort_button = QPushButton("ABORT ALL")
        self.abort_button.setStyleSheet(
            "QPushButton { "
            "background-color: #d32f2f; "
            "color: white; "
            "font-weight: bold; "
            "font-size: 12pt; "
            "padding: 10px 20px; "
            "}"
            "QPushButton:hover { "
            "background-color: #b71c1c; "
            "}"
        )
        self.abort_button.clicked.connect(self.on_abort)
        button_layout.addWidget(self.abort_button)

        layout.addLayout(button_layout)

    def get_policy(self) -> str:
        """Get selected conflict policy."""
        if self.rb_skip.isChecked():
            return "skip"
        elif self.rb_replace.isChecked():
            return "replace"
        elif self.rb_keep_both.isChecked():
            return "keep_both"
        return "skip"

    def on_apply(self) -> None:
        """Apply the selected policy for this file."""
        self.conflict_policy = self.get_policy()
        self.abort_all = False
        self.accept()

    def on_skip(self) -> None:
        """Skip this file and continue."""
        self.conflict_policy = "skip"
        self.abort_all = False
        self.accept()

    def on_abort(self) -> None:
        """Abort the entire operation."""
        self.abort_all = True
        self.reject()
