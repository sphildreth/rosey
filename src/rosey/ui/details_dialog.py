"""Dialog for displaying detailed information about a media item."""

from pathlib import Path

from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QDialog,
    QGridLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QMessageBox,
    QPushButton,
    QScrollArea,
    QSizePolicy,
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
        self.resize(1000, 800)  # Increased width for better text display
        # Make dialog resizable
        self.setMinimumSize(700, 600)

        self.init_ui()

    def init_ui(self) -> None:
        """Initialize the UI."""
        main_layout = QVBoxLayout(self)
        main_layout.setSpacing(15)

        # Create scroll area for content
        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        scroll_area.setFrameShape(QScrollArea.Shape.NoFrame)

        # Content widget
        content_widget = QWidget()
        layout = QVBoxLayout(content_widget)
        layout.setSpacing(15)

        # Basic Information
        basic_group = QGroupBox("Basic Information")
        basic_layout = QGridLayout(basic_group)
        basic_layout.setSpacing(10)
        basic_layout.setColumnStretch(1, 1)  # Allow value column to expand
        basic_group.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)

        row = 0
        self._add_grid_row(basic_layout, row, "Type:", self.item.kind.upper())
        row += 1
        self._add_grid_row(basic_layout, row, "Title:", self.item.title or "Unknown")
        row += 1

        if self.item.year:
            self._add_grid_row(basic_layout, row, "Year:", str(self.item.year))
            row += 1

        if self.item.kind == "episode":
            if self.item.season is not None:
                self._add_grid_row(basic_layout, row, "Season:", str(self.item.season))
                row += 1
            if self.item.episodes:
                episodes_str = ", ".join(str(e) for e in self.item.episodes)
                self._add_grid_row(basic_layout, row, "Episode(s):", episodes_str)
                row += 1
            if self.item.date:
                self._add_grid_row(basic_layout, row, "Date:", self.item.date)
                row += 1

        if self.item.part:
            self._add_grid_row(basic_layout, row, "Part:", str(self.item.part))
            row += 1

        # Add file size and modification date
        source_path = Path(self.item.source_path)
        if source_path.exists():
            # File size
            file_size = source_path.stat().st_size
            size_mb = file_size / (1024 * 1024)
            size_str = f"{size_mb / 1024:.2f} GB" if size_mb >= 1024 else f"{size_mb:.2f} MB"
            self._add_grid_row(basic_layout, row, "File Size:", size_str)
            row += 1

            # Modification date
            import datetime

            mod_time = source_path.stat().st_mtime
            mod_date = datetime.datetime.fromtimestamp(mod_time)
            mod_date_str = mod_date.strftime("%Y-%m-%d %H:%M:%S")
            self._add_grid_row(basic_layout, row, "Modified:", mod_date_str)

        layout.addWidget(basic_group)

        # File Information
        file_group = QGroupBox("File Information")
        file_layout = QGridLayout(file_group)
        file_layout.setSpacing(10)
        file_layout.setColumnStretch(1, 1)
        file_group.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)

        row = 0
        self._add_grid_row(file_layout, row, "Source Path:", self.item.source_path)
        row += 1
        self._add_grid_row(file_layout, row, "Destination:", self.destination)
        row += 1

        if self.item.sidecars:
            sidecars_str = "\n".join(self.item.sidecars)
            self._add_grid_row(file_layout, row, "Sidecar Files:", sidecars_str)

        layout.addWidget(file_group)

        # Metadata Information
        if self.item.nfo:
            metadata_group = QGroupBox("Metadata")
            metadata_layout = QGridLayout(metadata_group)
            metadata_layout.setSpacing(10)
            metadata_layout.setColumnStretch(1, 1)
            metadata_group.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)

            row = 0
            for key, value in self.item.nfo.items():
                if value:
                    self._add_grid_row(metadata_layout, row, f"{key}:", str(value))
                    row += 1

            layout.addWidget(metadata_group)

        # Confidence Score
        score_group = QGroupBox("Confidence Score")
        score_layout = QVBoxLayout(score_group)
        score_layout.setSpacing(10)
        score_group.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)

        confidence_label = QLabel(f"<b>Confidence: {self.score.confidence}%</b>")
        confidence_label.setStyleSheet(self._get_confidence_color(self.score.confidence))
        score_layout.addWidget(confidence_label)

        if self.score.reasons:
            reasons_text = QTextEdit()
            reasons_text.setReadOnly(True)
            reasons_text.setMaximumHeight(150)
            reasons_text.setStyleSheet("QTextEdit { font-size: 11pt; }")
            reasons_content = "Scoring Details:\n\n"
            for reason in self.score.reasons:
                reasons_content += f"• {reason}\n"
            reasons_text.setPlainText(reasons_content)
            score_layout.addWidget(reasons_text)

        layout.addWidget(score_group)

        # Media Group Information
        if self.group:
            group_group = QGroupBox("Media Group")
            group_layout = QGridLayout(group_group)
            group_layout.setSpacing(10)
            group_layout.setColumnStretch(1, 1)
            group_group.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)

            row = 0
            self._add_grid_row(group_layout, row, "Directory:", self.group.directory)
            row += 1
            self._add_grid_row(group_layout, row, "Group Type:", self.group.kind)
            row += 1
            self._add_grid_row(
                group_layout, row, "Total Videos:", str(len(self.group.primary_videos))
            )
            row += 1

            if self.group.directory_companions:
                companions_str = "\n".join(self.group.directory_companions)
                self._add_grid_row(group_layout, row, "Directory Companions:", companions_str)

            layout.addWidget(group_group)

        # Set the content widget in scroll area
        scroll_area.setWidget(content_widget)
        main_layout.addWidget(scroll_area)

        # Buttons
        button_layout = QHBoxLayout()
        button_layout.setSpacing(10)

        # Delete button
        self.delete_btn = QPushButton("Delete File")
        self.delete_btn.setStyleSheet(
            "QPushButton { background-color: #d32f2f; color: white; padding: 8px 16px; font-size: 11pt; }"
        )
        self.delete_btn.clicked.connect(self.on_delete_file)
        button_layout.addWidget(self.delete_btn)

        button_layout.addStretch()

        # Close button
        close_btn = QPushButton("Close")
        close_btn.setStyleSheet("QPushButton { padding: 8px 16px; font-size: 11pt; }")
        close_btn.clicked.connect(self.accept)
        close_btn.setDefault(True)
        button_layout.addWidget(close_btn)

        main_layout.addLayout(button_layout)

    def _add_grid_row(self, layout: QGridLayout, row: int, label: str, value: str) -> None:
        """Add an information row with label and value in a grid layout."""
        # Label
        label_widget = QLabel(f"<b>{label}</b>")
        label_widget.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignRight)
        label_widget.setStyleSheet("QLabel { padding: 5px 10px 5px 0px; font-size: 11pt; }")
        layout.addWidget(label_widget, row, 0)

        # Value
        value_widget = QLabel(value)
        value_widget.setWordWrap(True)
        value_widget.setTextInteractionFlags(Qt.TextInteractionFlag.TextSelectableByMouse)
        value_widget.setAlignment(Qt.AlignmentFlag.AlignTop | Qt.AlignmentFlag.AlignLeft)
        value_widget.setStyleSheet("QLabel { padding: 5px 0px 5px 10px; font-size: 11pt; }")
        value_widget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)
        value_widget.setMinimumWidth(400)
        layout.addWidget(value_widget, row, 1)

    def _add_info_row(self, layout: QVBoxLayout, label: str, value: str) -> None:
        """Add an information row with label and value (deprecated, kept for compatibility)."""
        label_widget = QLabel(f"<b>{label}</b>")
        layout.addWidget(label_widget)

        value_widget = QLabel(value)
        value_widget.setWordWrap(True)
        value_widget.setTextInteractionFlags(Qt.TextInteractionFlag.TextSelectableByMouse)
        value_widget.setStyleSheet(
            "QLabel { padding-left: 10px; padding-bottom: 8px; font-size: 11pt; }"
        )
        value_widget.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Preferred)
        value_widget.setMinimumWidth(200)
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
