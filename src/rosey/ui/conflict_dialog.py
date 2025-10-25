"""Conflict resolution dialog."""

from PySide6.QtWidgets import (
    QDialog,
    QDialogButtonBox,
    QLabel,
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

        # Buttons
        button_box = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel
        )
        button_box.accepted.connect(self.accept)
        button_box.rejected.connect(self.reject)
        layout.addWidget(button_box)

    def get_policy(self) -> str:
        """Get selected conflict policy."""
        if self.rb_skip.isChecked():
            return "skip"
        elif self.rb_replace.isChecked():
            return "replace"
        elif self.rb_keep_both.isChecked():
            return "keep_both"
        return "skip"

    def accept(self) -> None:
        """Handle dialog acceptance."""
        self.conflict_policy = self.get_policy()
        super().accept()
