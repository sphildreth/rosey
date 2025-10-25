"""Minimal UI for Rosey."""

from PySide6.QtCore import QRunnable, Qt, Signal, Slot
from PySide6.QtWidgets import (
    QAbstractItemView,
    QApplication,
    QHBoxLayout,
    QHeaderView,
    QMainWindow,
    QMenu,
    QPushButton,
    QSplitter,
    QTableWidget,
    QTableWidgetItem,
    QTextEdit,
    QTreeWidget,
    QTreeWidgetItem,
    QVBoxLayout,
    QWidget,
)

from rosey.config import get_config_path, load_config, save_config
from rosey.identifier import identify_file
from rosey.planner import plan_path
from rosey.providers import ProviderManager
from rosey.scanner import scan_directory
from rosey.scorer import score_identification
from rosey.ui.settings_dialog import SettingsDialog


class ScanWorker(QRunnable):
    """Background worker for scanning."""

    class Signals(QWidget):
        """Signals for scan worker."""

        progress = Signal(str)
        finished = Signal(list)

    def __init__(self, source_path: str, max_workers: int = 8) -> None:
        super().__init__()
        self.source_path = source_path
        self.max_workers = max_workers
        self.signals = self.Signals()

    def run(self) -> None:
        """Run the scan."""
        self.signals.progress.emit(f"Scanning {self.source_path}...")

        try:
            results = scan_directory(self.source_path, max_workers=self.max_workers)
            video_results = [r for r in results if r.is_video and not r.error]

            items = []
            for scan_result in video_results:
                self.signals.progress.emit(f"Identifying {scan_result.path}...")

                # Identify
                ident_result = identify_file(scan_result.path)

                # Score
                score = score_identification(ident_result)

                # Plan destination (using dummy paths for now)
                destination = plan_path(
                    ident_result.item,
                    movies_root="/media/movies",
                    tv_root="/media/tv",
                )

                items.append(
                    {
                        "item": ident_result.item,
                        "score": score,
                        "destination": destination,
                    }
                )

            self.signals.finished.emit(items)
        except Exception as e:
            self.signals.progress.emit(f"Error: {e}")


class MainWindow(QMainWindow):
    """Main window for Rosey."""

    def __init__(self) -> None:
        super().__init__()
        self.items: list[dict] = []

        # Load configuration
        self.config = load_config()

        # Initialize provider manager
        cache_dir = get_config_path().parent / "cache"
        self.provider_manager = ProviderManager(
            cache_dir=cache_dir,
            cache_ttl_days=self.config.providers.cache_ttl_days,
            enabled=self.config.identification.use_online_providers,
        )

        # Configure providers if enabled
        if self.config.identification.use_online_providers:
            if self.config.providers.tmdb_api_key:
                self.provider_manager.configure_tmdb(
                    self.config.providers.tmdb_api_key,
                    self.config.providers.tmdb_language,
                    self.config.providers.tmdb_region,
                )
            if self.config.providers.tvdb_api_key:
                self.provider_manager.configure_tvdb(
                    self.config.providers.tvdb_api_key,
                    self.config.providers.tvdb_language,
                )

        self.init_ui()

    def closeEvent(self, event) -> None:  # type: ignore[no-untyped-def]  # noqa: N802
        """Handle window close event."""
        self.provider_manager.close()
        event.accept()

    def init_ui(self) -> None:
        """Initialize the UI."""
        self.setWindowTitle("Rosey - Media File Organizer")
        self.setGeometry(100, 100, 1200, 800)

        # Central widget
        central = QWidget()
        self.setCentralWidget(central)

        # Main layout
        layout = QVBoxLayout(central)

        # Toolbar
        toolbar = QHBoxLayout()

        self.btn_scan = QPushButton("Scan")
        self.btn_scan.clicked.connect(self.on_scan)
        toolbar.addWidget(self.btn_scan)

        self.btn_settings = QPushButton("Settings")
        self.btn_settings.clicked.connect(self.on_settings)
        toolbar.addWidget(self.btn_settings)

        self.btn_select_green = QPushButton("Select All Green")
        self.btn_select_green.clicked.connect(self.on_select_green)
        toolbar.addWidget(self.btn_select_green)

        self.btn_clear = QPushButton("Clear Selection")
        self.btn_clear.clicked.connect(self.on_clear_selection)
        toolbar.addWidget(self.btn_clear)

        toolbar.addStretch()

        self.filter_all = QPushButton("All")
        self.filter_all.clicked.connect(lambda: self.on_filter("all"))
        toolbar.addWidget(self.filter_all)

        self.filter_green = QPushButton("Green")
        self.filter_green.clicked.connect(lambda: self.on_filter("green"))
        toolbar.addWidget(self.filter_green)

        self.filter_yellow = QPushButton("Yellow")
        self.filter_yellow.clicked.connect(lambda: self.on_filter("yellow"))
        toolbar.addWidget(self.filter_yellow)

        self.filter_red = QPushButton("Red")
        self.filter_red.clicked.connect(lambda: self.on_filter("red"))
        toolbar.addWidget(self.filter_red)

        layout.addLayout(toolbar)

        # Main splitter (horizontal: tree on left, table on right)
        main_splitter = QSplitter(Qt.Orientation.Horizontal)

        # Tree widget (left)
        self.tree = QTreeWidget()
        self.tree.setHeaderLabel("Media Type")
        self.tree.itemSelectionChanged.connect(self.on_tree_selection)
        self.tree.setContextMenuPolicy(Qt.ContextMenuPolicy.CustomContextMenu)
        self.tree.customContextMenuRequested.connect(self.on_tree_context_menu)

        # Add root items
        self.tree_movies = QTreeWidgetItem(self.tree, ["Movies"])
        self.tree_tv = QTreeWidgetItem(self.tree, ["TV Shows"])
        self.tree_unknown = QTreeWidgetItem(self.tree, ["Unknown"])

        self.tree.expandAll()
        main_splitter.addWidget(self.tree)

        # Table widget (right)
        self.table = QTableWidget()
        self.table.setColumnCount(5)
        self.table.setHorizontalHeaderLabels(["✓", "Type", "Name", "Confidence", "Destination"])
        self.table.setSelectionBehavior(QAbstractItemView.SelectionBehavior.SelectRows)
        self.table.setEditTriggers(QAbstractItemView.EditTrigger.NoEditTriggers)

        # Resize columns
        header = self.table.horizontalHeader()
        header.setSectionResizeMode(0, QHeaderView.ResizeMode.ResizeToContents)
        header.setSectionResizeMode(1, QHeaderView.ResizeMode.ResizeToContents)
        header.setSectionResizeMode(2, QHeaderView.ResizeMode.Stretch)
        header.setSectionResizeMode(3, QHeaderView.ResizeMode.ResizeToContents)
        header.setSectionResizeMode(4, QHeaderView.ResizeMode.Stretch)

        main_splitter.addWidget(self.table)
        main_splitter.setStretchFactor(1, 3)

        # Vertical splitter (top: main area, bottom: activity log)
        vert_splitter = QSplitter(Qt.Orientation.Vertical)
        vert_splitter.addWidget(main_splitter)

        # Activity log (bottom)
        self.activity_log = QTextEdit()
        self.activity_log.setReadOnly(True)
        self.activity_log.setMaximumHeight(150)
        vert_splitter.addWidget(self.activity_log)

        layout.addWidget(vert_splitter)

        # Status bar
        self.statusBar().showMessage("Ready")

        # Add seed data
        self.add_seed_data()

    def add_seed_data(self) -> None:
        """Add seed data for UI testing."""
        seed_items = [
            {
                "item": type(
                    "MediaItem",
                    (),
                    {
                        "kind": "episode",
                        "title": "The Office",
                        "season": 2,
                        "episodes": [1],
                        "source_path": "/source/The Office S02E01.mkv",
                    },
                )(),
                "score": type("Score", (), {"confidence": 90, "reasons": []})(),
                "destination": "/media/tv/The Office/Season 02/The Office - S02E01.mkv",
            },
            {
                "item": type(
                    "MediaItem",
                    (),
                    {
                        "kind": "movie",
                        "title": "The Matrix",
                        "year": 1999,
                        "source_path": "/source/The Matrix (1999).mkv",
                    },
                )(),
                "score": type("Score", (), {"confidence": 85, "reasons": []})(),
                "destination": "/media/movies/The Matrix (1999)/The Matrix (1999).mkv",
            },
            {
                "item": type(
                    "MediaItem",
                    (),
                    {
                        "kind": "episode",
                        "title": "Breaking Bad",
                        "season": 1,
                        "episodes": [1],
                        "source_path": "/source/Breaking Bad S01E01.mkv",
                    },
                )(),
                "score": type("Score", (), {"confidence": 55, "reasons": []})(),
                "destination": "/media/tv/Breaking Bad/Season 01/Breaking Bad - S01E01.mkv",
            },
        ]

        self.items = seed_items
        self.populate_table(seed_items)
        self.activity_log.append("Loaded 3 seed items")

    def populate_table(self, items: list) -> None:
        """Populate table with items."""
        self.table.setRowCount(len(items))

        for row, result in enumerate(items):
            item = result["item"]
            score = result["score"]
            dest = result["destination"]

            # Checkbox
            check_item = QTableWidgetItem()
            check_item.setFlags(Qt.ItemFlag.ItemIsUserCheckable | Qt.ItemFlag.ItemIsEnabled)
            check_item.setCheckState(Qt.CheckState.Unchecked)
            self.table.setItem(row, 0, check_item)

            # Type
            type_item = QTableWidgetItem(item.kind.upper())
            self.table.setItem(row, 1, type_item)

            # Name
            if item.kind == "movie":
                name = f"{item.title} ({getattr(item, 'year', '?')})"
            elif item.kind == "episode":
                if hasattr(item, "episodes") and item.episodes:
                    name = f"{item.title} - S{item.season:02d}E{item.episodes[0]:02d}"
                else:
                    name = item.title
            else:
                name = item.title or "Unknown"

            name_item = QTableWidgetItem(name)
            self.table.setItem(row, 2, name_item)

            # Confidence
            conf_item = QTableWidgetItem(f"{score.confidence}%")

            # Color code by confidence
            if score.confidence >= 70:
                conf_item.setBackground(Qt.GlobalColor.green)
            elif score.confidence >= 40:
                conf_item.setBackground(Qt.GlobalColor.yellow)
            else:
                conf_item.setBackground(Qt.GlobalColor.red)

            self.table.setItem(row, 3, conf_item)

            # Destination
            dest_item = QTableWidgetItem(dest)
            self.table.setItem(row, 4, dest_item)

    @Slot()
    def on_scan(self) -> None:
        """Handle scan button click."""
        # For demo, just use seed data
        self.activity_log.append("Scan button clicked (seed data mode)")
        self.statusBar().showMessage("Scan completed - using seed data")

    @Slot()
    def on_select_green(self) -> None:
        """Select all items with green confidence."""
        count = 0
        for row in range(self.table.rowCount()):
            conf_item = self.table.item(row, 3)
            if conf_item:
                conf_text = conf_item.text().rstrip("%")
                confidence = int(conf_text)

                if confidence >= 70:
                    check_item = self.table.item(row, 0)
                    if check_item:
                        check_item.setCheckState(Qt.CheckState.Checked)
                        count += 1

        self.activity_log.append(f"Selected {count} items with green confidence (>=70%)")
        self.statusBar().showMessage(f"Selected {count} green items")

    @Slot()
    def on_clear_selection(self) -> None:
        """Clear all selections."""
        for row in range(self.table.rowCount()):
            check_item = self.table.item(row, 0)
            if check_item:
                check_item.setCheckState(Qt.CheckState.Unchecked)

        self.activity_log.append("Cleared all selections")
        self.statusBar().showMessage("Selections cleared")

    def on_filter(self, filter_type: str) -> None:
        """Filter table by confidence level."""
        self.activity_log.append(f"Filtering: {filter_type}")

        # Show all items first
        for row in range(self.table.rowCount()):
            self.table.setRowHidden(row, False)

        if filter_type != "all":
            for row in range(self.table.rowCount()):
                conf_item = self.table.item(row, 3)
                if conf_item:
                    conf_text = conf_item.text().rstrip("%")
                    confidence = int(conf_text)

                    hide = False
                    if (
                        filter_type == "green"
                        and confidence < 70
                        or filter_type == "yellow"
                        and not (40 <= confidence < 70)
                        or filter_type == "red"
                        and confidence >= 40
                    ):
                        hide = True

                    self.table.setRowHidden(row, hide)

        self.statusBar().showMessage(f"Filter: {filter_type}")

    @Slot()
    def on_tree_selection(self) -> None:
        """Handle tree selection change."""
        selected = self.tree.selectedItems()
        if not selected:
            return

        item_text = selected[0].text(0)
        self.activity_log.append(f"Tree selection: {item_text}")

        # Filter table based on tree selection
        for row in range(self.table.rowCount()):
            type_item = self.table.item(row, 1)
            if type_item:
                item_type = type_item.text()

                hide = False
                if (
                    item_text == "Movies"
                    and item_type != "MOVIE"
                    or item_text == "TV Shows"
                    and item_type != "EPISODE"
                    or item_text == "Unknown"
                    and item_type != "UNKNOWN"
                ):
                    hide = True

                self.table.setRowHidden(row, hide)

    @Slot()
    def on_settings(self) -> None:
        """Handle settings button click."""
        dialog = SettingsDialog(self.config, self)
        if dialog.exec():
            # Save updated config
            self.config = dialog.get_config()
            save_config(self.config)

            # Update provider manager
            self.provider_manager.enabled = self.config.identification.use_online_providers

            if self.config.identification.use_online_providers:
                if self.config.providers.tmdb_api_key:
                    self.provider_manager.configure_tmdb(
                        self.config.providers.tmdb_api_key,
                        self.config.providers.tmdb_language,
                        self.config.providers.tmdb_region,
                    )
                if self.config.providers.tvdb_api_key:
                    self.provider_manager.configure_tvdb(
                        self.config.providers.tvdb_api_key,
                        self.config.providers.tvdb_language,
                    )

            self.activity_log.append("Settings saved")
            self.statusBar().showMessage("Settings updated")

    @Slot()
    def on_tree_context_menu(self, position) -> None:  # type: ignore[no-untyped-def]
        """Handle tree context menu request."""
        item = self.tree.itemAt(position)
        if not item:
            return

        menu = QMenu(self)
        discover_action = menu.addAction("Discover Metadata...")

        # Disable if providers not enabled
        if not self.config.identification.use_online_providers:
            discover_action.setEnabled(False)
            discover_action.setToolTip("Enable online providers in Settings to use this feature")

        action = menu.exec(self.tree.viewport().mapToGlobal(position))

        if action == discover_action and self.config.identification.use_online_providers:
            self.on_discover(item)

    def on_discover(self, tree_item: QTreeWidgetItem) -> None:
        """Handle discover metadata action.

        Args:
            tree_item: Selected tree item
        """
        item_text = tree_item.text(0)
        self.activity_log.append(f"Discovering metadata for: {item_text}")
        self.statusBar().showMessage(f"Discovering metadata for {item_text}...")

        # In a real implementation, this would run in a background thread
        # For now, we'll just show a placeholder message

        # TODO: Implement background discovery task that:
        # 1. Finds all items in the selected tree node
        # 2. Queries provider_manager for each item
        # 3. Updates scores and metadata
        # 4. Refreshes the table

        self.activity_log.append("Discover: Feature stub - provider calls would happen here")
        self.statusBar().showMessage("Ready")


def main() -> int:
    """Run the UI."""
    app = QApplication([])
    window = MainWindow()
    window.show()
    return app.exec()


if __name__ == "__main__":
    main()
