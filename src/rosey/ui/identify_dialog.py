"""Dialog for manual identification of media items."""

from typing import Any

from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QComboBox,
    QDialog,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QListWidget,
    QListWidgetItem,
    QMessageBox,
    QPushButton,
    QVBoxLayout,
    QWidget,
)


class IdentifyDialog(QDialog):
    """Dialog for identifying media items manually or via online providers."""

    def __init__(
        self,
        provider_manager: Any,
        initial_title: str = "",
        initial_type: str = "movie",
        parent: QWidget | None = None,
    ) -> None:
        super().__init__(parent)
        self.pm = provider_manager
        self.selected_result = None
        self.setWindowTitle("Identify Media")
        self.setModal(True)
        self.resize(500, 600)

        layout = QVBoxLayout(self)

        # Type selection
        type_layout = QHBoxLayout()
        type_layout.addWidget(QLabel("Media Type:"))
        self.type_combo = QComboBox()
        self.type_combo.addItems(["Movie", "TV Show"])
        if initial_type == "episode":
            self.type_combo.setCurrentText("TV Show")
        self.type_combo.currentTextChanged.connect(self.on_type_changed)
        type_layout.addWidget(self.type_combo)
        layout.addLayout(type_layout)

        # Search section
        search_group = QGroupBox("Online Search")
        search_layout = QVBoxLayout(search_group)

        search_form = QHBoxLayout()
        search_form.addWidget(QLabel("Title:"))
        self.title_edit = QLineEdit(initial_title)
        search_form.addWidget(self.title_edit)
        search_form.addWidget(QLabel("Year:"))
        self.year_edit = QLineEdit()
        search_form.addWidget(self.year_edit)
        layout.addLayout(search_form)

        self.search_btn = QPushButton("Search")
        self.search_btn.clicked.connect(self.on_search)
        if not self.pm.enabled:
            self.search_btn.setEnabled(False)
            self.search_btn.setToolTip("Online providers not enabled")
        search_layout.addWidget(self.search_btn)

        self.results_list = QListWidget()
        self.results_list.itemDoubleClicked.connect(self.on_result_selected)
        search_layout.addWidget(self.results_list)

        layout.addWidget(search_group)

        # Manual entry section
        manual_group = QGroupBox("Manual Entry")
        manual_layout = QVBoxLayout(manual_group)

        manual_layout.addWidget(QLabel("Title:"))
        self.manual_title = QLineEdit()
        manual_layout.addWidget(self.manual_title)

        manual_layout.addWidget(QLabel("Year:"))
        self.manual_year = QLineEdit()
        manual_layout.addWidget(self.manual_year)

        # TV-specific fields
        self.season_label = QLabel("Season (TV Shows only):")
        manual_layout.addWidget(self.season_label)
        self.season_edit = QLineEdit()
        manual_layout.addWidget(self.season_edit)

        self.episode_label = QLabel("Episode (TV Shows only):")
        manual_layout.addWidget(self.episode_label)
        self.episode_edit = QLineEdit()
        manual_layout.addWidget(self.episode_edit)

        layout.addWidget(manual_group)

        # Initially hide season/episode for TV Shows since we're identifying the show
        self.on_type_changed(self.type_combo.currentText())

        # Buttons
        buttons = QHBoxLayout()
        self.ok_btn = QPushButton("OK")
        self.ok_btn.clicked.connect(self.accept)
        buttons.addWidget(self.ok_btn)

        cancel_btn = QPushButton("Cancel")
        cancel_btn.clicked.connect(self.reject)
        buttons.addWidget(cancel_btn)

        layout.addLayout(buttons)

    def on_search(self) -> None:
        """Search online providers for matches."""
        if not self.pm.enabled:
            QMessageBox.information(
                self,
                "Providers Disabled",
                "Online providers are not enabled. Use manual entry instead.",
            )
            return
        title = self.title_edit.text().strip()
        year_str = self.year_edit.text().strip()
        year = int(year_str) if year_str.isdigit() else None
        media_type = "movie" if self.type_combo.currentText() == "Movie" else "tv"

        if not title:
            QMessageBox.warning(self, "Input Required", "Please enter a title to search.")
            return

        self.results_list.clear()
        self.search_btn.setEnabled(False)
        self.search_btn.setText("Searching...")

        try:
            if media_type == "movie":
                results = self.pm.search_movie(title, year)
            else:
                results = self.pm.search_tv(title, year)

            for result in results[:10]:  # Limit to 10 results
                title = result.get("title", result.get("name", "Unknown"))
                date = result.get("release_date", result.get("first_air_date", ""))[:4] or "N/A"
                display_text = f"{title} ({date})"
                item = QListWidgetItem(display_text)
                item.setData(Qt.ItemDataRole.UserRole, result)
                self.results_list.addItem(item)

        except Exception as e:
            QMessageBox.warning(self, "Search Error", f"Failed to search: {str(e)}")
        finally:
            self.search_btn.setEnabled(True)
            self.search_btn.setText("Search")

    def on_type_changed(self, media_type: str) -> None:
        """Show/hide season and episode fields based on media type."""
        is_tv = media_type == "TV Show"
        self.season_label.setVisible(is_tv)
        self.season_edit.setVisible(is_tv)
        self.episode_label.setVisible(is_tv)
        self.episode_edit.setVisible(is_tv)

    def on_result_selected(self, item: QListWidgetItem) -> None:
        """Handle selection of a search result."""
        self.selected_result = item.data(Qt.ItemDataRole.UserRole)
        if self.selected_result:
            # Pre-fill manual fields with selected result
            self.manual_title.setText(
                self.selected_result.get("title", self.selected_result.get("name", ""))
            )
            date = self.selected_result.get(
                "release_date", self.selected_result.get("first_air_date", "")
            )
            if date:
                self.manual_year.setText(date[:4])

    def get_result(self) -> dict:
        """Get the identification result."""
        is_tv = self.type_combo.currentText() == "TV Show"
        if self.selected_result:
            # Return provider result
            result = self.selected_result.copy()
            result["type"] = "episode" if is_tv else "movie"
            return result
        else:
            # Return manual entry
            result = {
                "title": self.manual_title.text(),
                "year": self.manual_year.text(),
                "type": "episode" if is_tv else "movie",
            }
            # Only include season/episode for TV if provided (though hidden in this context)
            if is_tv and self.season_edit.text():
                result["season"] = self.season_edit.text()
            if is_tv and self.episode_edit.text():
                result["episode"] = self.episode_edit.text()
            return result
