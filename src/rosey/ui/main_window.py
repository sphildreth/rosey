"""Minimal UI for Rosey."""

from PySide6.QtCore import QRunnable, Qt, Signal, Slot
from PySide6.QtWidgets import (
    QAbstractItemView,
    QApplication,
    QHBoxLayout,
    QHeaderView,
    QMainWindow,
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

from rosey.identifier import identify_file
from rosey.planner import plan_path
from rosey.scanner import scan_directory
from rosey.scorer import score_identification


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

                items.append({
                    "item": ident_result.item,
                    "score": score,
                    "destination": destination,
                })

            self.signals.finished.emit(items)
        except Exception as e:
            self.signals.progress.emit(f"Error: {e}")


class MainWindow(QMainWindow):
    """Main window for Rosey."""

    def __init__(self) -> None:
        super().__init__()
        self.items: list[dict] = []
        self.init_ui()

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

        # Add root items
        self.tree_movies = QTreeWidgetItem(self.tree, ["Movies"])
        self.tree_tv = QTreeWidgetItem(self.tree, ["TV Shows"])
        self.tree_unknown = QTreeWidgetItem(self.tree, ["Unknown"])

        self.tree.expandAll()
        main_splitter.addWidget(self.tree)

        # Table widget (right)
        self.table = QTableWidget()
        self.table.setColumnCount(5)
        self.table.setHorizontalHeaderLabels(
            ["✓", "Type", "Name", "Confidence", "Destination"]
        )
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
                    if filter_type == "green" and confidence < 70 or filter_type == "yellow" and not (40 <= confidence < 70) or filter_type == "red" and confidence >= 40:
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
                if item_text == "Movies" and item_type != "MOVIE" or item_text == "TV Shows" and item_type != "EPISODE" or item_text == "Unknown" and item_type != "UNKNOWN":
                    hide = True

                self.table.setRowHidden(row, hide)


def main() -> int:
    """Run the UI."""
    app = QApplication([])
    window = MainWindow()
    window.show()
    return app.exec()


if __name__ == "__main__":
    main()
