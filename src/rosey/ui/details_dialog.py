"""Dialog for displaying detailed information about a media item."""

from pathlib import Path

from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QDialog,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QMessageBox,
    QPushButton,
    QTextEdit,
    QVBoxLayout,
    QWidget,
)


class DetailsDialog(QDialog):
    """Dialog showing detailed information about a media item."""

    def __init__(
        self,
        item_dict: dict,
        parent: QWidget | None = None,
    ) -> None:
        """
        Initialize the details dialog.

        Args:
            item_dict: Dictionary containing 'item', 'score', 'destination', 'group'
            parent: Parent widget
        """
        super().__init__(parent)
        self.item_dict = item_dict
        self.item = item_dict["item"]
        self.score = item_dict["score"]
        self.destination = item_dict["destination"]
        self.group = item_dict.get("group")
        self.file_deleted = False

        self.setWindowTitle("Media Item Details")
        self.setModal(True)
        self.resize(800, 700)
        # Make dialog resizable
        self.setMinimumSize(600, 500)

        self.init_ui()

    def init_ui(self) -> None:
        """Initialize the UI."""
        layout = QVBoxLayout(self)

        # Basic Information
        basic_group = QGroupBox("Basic Information")
        basic_layout = QVBoxLayout(basic_group)

        self._add_info_row(basic_layout, "Type:", self.item.kind.upper())
        self._add_info_row(basic_layout, "Title:", self.item.title or "Unknown")

        if self.item.year:
            self._add_info_row(basic_layout, "Year:", str(self.item.year))

        if self.item.kind == "episode":
            if self.item.season is not None:
                self._add_info_row(basic_layout, "Season:", str(self.item.season))
            if self.item.episodes:
                episodes_str = ", ".join(str(e) for e in self.item.episodes)
                self._add_info_row(basic_layout, "Episode(s):", episodes_str)
            if self.item.date:
                self._add_info_row(basic_layout, "Date:", self.item.date)

        if self.item.part:
            self._add_info_row(basic_layout, "Part:", str(self.item.part))

        # Add file size and modification date
        source_path = Path(self.item.source_path)
        if source_path.exists():
            # File size
            file_size = source_path.stat().st_size
            size_mb = file_size / (1024 * 1024)
            size_str = f"{size_mb / 1024:.2f} GB" if size_mb >= 1024 else f"{size_mb:.2f} MB"
            self._add_info_row(basic_layout, "File Size:", size_str)

            # Modification date
            import datetime

            mod_time = source_path.stat().st_mtime
            mod_date = datetime.datetime.fromtimestamp(mod_time)
            mod_date_str = mod_date.strftime("%Y-%m-%d %H:%M:%S")
            self._add_info_row(basic_layout, "Modified:", mod_date_str)

        layout.addWidget(basic_group)

        # File Information
        file_group = QGroupBox("File Information")
        file_layout = QVBoxLayout(file_group)

        self._add_info_row(file_layout, "Source Path:", self.item.source_path)
        self._add_info_row(file_layout, "Destination:", self.destination)

        if self.item.sidecars:
            sidecars_str = "\n".join(self.item.sidecars)
            self._add_info_row(file_layout, "Sidecar Files:", sidecars_str)

        layout.addWidget(file_group)

        # Metadata Information
        if self.item.nfo:
            metadata_group = QGroupBox("Metadata")
            metadata_layout = QVBoxLayout(metadata_group)

            for key, value in self.item.nfo.items():
                if value:
                    self._add_info_row(metadata_layout, f"{key}:", str(value))

            layout.addWidget(metadata_group)

        # Confidence Score
        score_group = QGroupBox("Confidence Score")
        score_layout = QVBoxLayout(score_group)

        confidence_label = QLabel(f"<b>Confidence: {self.score.confidence}%</b>")
        confidence_label.setStyleSheet(self._get_confidence_color(self.score.confidence))
        score_layout.addWidget(confidence_label)

        if self.score.reasons:
            reasons_text = QTextEdit()
            reasons_text.setReadOnly(True)
            reasons_text.setMaximumHeight(150)
            reasons_content = "Scoring Details:\n\n"
            for reason in self.score.reasons:
                reasons_content += f"• {reason}\n"
            reasons_text.setPlainText(reasons_content)
            score_layout.addWidget(reasons_text)

        layout.addWidget(score_group)

        # Media Group Information
        if self.group:
            group_group = QGroupBox("Media Group")
            group_layout = QVBoxLayout(group_group)

            self._add_info_row(group_layout, "Directory:", self.group.directory)
            self._add_info_row(group_layout, "Group Type:", self.group.kind)
            self._add_info_row(group_layout, "Total Videos:", str(len(self.group.primary_videos)))

            if self.group.directory_companions:
                companions_str = "\n".join(self.group.directory_companions)
                self._add_info_row(group_layout, "Directory Companions:", companions_str)

            layout.addWidget(group_group)

        # Buttons
        button_layout = QHBoxLayout()

        # Delete button
        self.delete_btn = QPushButton("Delete File")
        self.delete_btn.setStyleSheet("QPushButton { background-color: #d32f2f; color: white; }")
        self.delete_btn.clicked.connect(self.on_delete_file)
        button_layout.addWidget(self.delete_btn)

        button_layout.addStretch()

        # Close button
        close_btn = QPushButton("Close")
        close_btn.clicked.connect(self.accept)
        close_btn.setDefault(True)
        button_layout.addWidget(close_btn)

        layout.addLayout(button_layout)

    def _add_info_row(self, layout: QVBoxLayout, label: str, value: str) -> None:
        """Add an information row with label and value."""
        label_widget = QLabel(f"<b>{label}</b>")
        layout.addWidget(label_widget)

        value_widget = QLabel(value)
        value_widget.setWordWrap(True)
        value_widget.setTextInteractionFlags(Qt.TextInteractionFlag.TextSelectableByMouse)
        value_widget.setStyleSheet("QLabel { padding-left: 10px; padding-bottom: 8px; }")
        layout.addWidget(value_widget)

    def _get_confidence_color(self, confidence: int) -> str:
        """Get color styling based on confidence level."""
        if confidence >= 70:
            return "QLabel { color: green; font-size: 14pt; }"
        elif confidence >= 40:
            return "QLabel { color: orange; font-size: 14pt; }"
        else:
            return "QLabel { color: red; font-size: 14pt; }"

    def on_delete_file(self) -> None:
        """Handle file deletion."""
        source_path = Path(self.item.source_path)

        # Confirmation dialog
        reply = QMessageBox.question(
            self,
            "Confirm Deletion",
            f"Are you sure you want to delete this file?\n\n{source_path}\n\n"
            f"This action cannot be undone.",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
            QMessageBox.StandardButton.No,
        )

        if reply != QMessageBox.StandardButton.Yes:
            return

        try:
            # Delete the main file
            if source_path.exists():
                source_path.unlink()

            # Delete sidecar files
            deleted_sidecars = []
            for sidecar in self.item.sidecars:
                sidecar_path = Path(sidecar)
                if sidecar_path.exists():
                    sidecar_path.unlink()
                    deleted_sidecars.append(sidecar)

            self.file_deleted = True

            # Show success message
            msg = f"File deleted successfully:\n{source_path}"
            if deleted_sidecars:
                msg += f"\n\nAlso deleted {len(deleted_sidecars)} sidecar file(s)"

            QMessageBox.information(self, "File Deleted", msg)

            # Close the dialog
            self.accept()

        except Exception as e:
            QMessageBox.critical(
                self,
                "Deletion Failed",
                f"Failed to delete file:\n{source_path}\n\nError: {str(e)}",
            )
