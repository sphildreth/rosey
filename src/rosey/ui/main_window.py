"""Minimal UI for Rosey."""

import contextlib
from pathlib import Path
from typing import Literal, cast

from PySide6.QtCore import QRunnable, Qt, QThreadPool, Signal, Slot
from PySide6.QtGui import QTextCursor
from PySide6.QtWidgets import (
    QAbstractItemView,
    QApplication,
    QHBoxLayout,
    QHeaderView,
    QMainWindow,
    QMenu,
    QPushButton,
    QSplitter,
    QStyle,
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
from rosey.models import IdentificationResult
from rosey.mover import move_with_sidecars
from rosey.planner import plan_path
from rosey.providers import ProviderManager
from rosey.scanner import scan_directory
from rosey.scorer import score_identification
from rosey.ui.identify_dialog import IdentifyDialog
from rosey.ui.progress_dialog import ProgressDialog
from rosey.ui.settings_dialog import SettingsDialog

# Type alias for mover conflict policy (excludes "ask")
MoverPolicy = Literal["skip", "replace", "keep_both"]


class ScanWorker(QRunnable):
    """Background worker for scanning."""

    class Signals(QWidget):
        """Signals for scan worker."""

        progress = Signal(str)
        finished = Signal(list)

    def __init__(
        self,
        source_path: str,
        *,
        max_workers: int = 8,
        follow_symlinks: bool = False,
        movies_root: str = "",
        tv_root: str = "",
    ) -> None:
        super().__init__()
        self.source_path = source_path
        self.max_workers = max_workers
        self.follow_symlinks = follow_symlinks
        self.movies_root = movies_root
        self.tv_root = tv_root
        self.signals = self.Signals()

    def run(self) -> None:
        """Run the scan."""
        self.signals.progress.emit(f"Scanning {self.source_path}...")

        try:
            results = scan_directory(
                self.source_path,
                max_workers=self.max_workers,
                follow_symlinks=self.follow_symlinks,
            )
            video_results = [r for r in results if r.is_video and not r.error]

            items = []
            for scan_result in video_results:
                self.signals.progress.emit(f"Identifying {scan_result.path}...")

                # Identify
                ident_result = identify_file(scan_result.path)

                # Score
                score = score_identification(ident_result)

                # Plan destination using configured roots
                destination = plan_path(
                    ident_result.item,
                    movies_root=self.movies_root,
                    tv_root=self.tv_root,
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


class DiscoverWorker(QRunnable):
    """Background worker for provider metadata discovery."""

    class Signals(QWidget):
        progress = Signal(str)
        finished = Signal(list)

    def __init__(self, items: list[dict], provider_manager: ProviderManager) -> None:
        super().__init__()
        self.items = items
        self.pm = provider_manager
        self.signals = self.Signals()

    def run(self) -> None:
        updated: list[dict] = []
        total = len(self.items)
        for idx, result in enumerate(self.items, start=1):
            item = result["item"]
            self.signals.progress.emit(f"Discover {idx}/{total}: {item.title or item.source_path}")
            try:
                # Movies: look up TMDB by title/year
                if item.kind == "movie":
                    if self.pm.enabled and not (item.nfo.get("tmdbid") if item.nfo else None):
                        title = item.title or ""
                        year = item.year
                        movies = self.pm.search_movie(title, year)
                        if movies:
                            m = movies[0]
                            tmdbid = str(m.get("id"))
                            item.nfo["tmdbid"] = tmdbid
                            # Update normalized title/year if missing
                            item.title = item.title or m.get("title") or m.get("name")
                            item.year = item.year or (
                                (m.get("release_date") or "").split("-")[0] or None
                            )
                # Episodes: look up TV show and optionally episode title
                elif (
                    item.kind == "episode"
                    and self.pm.enabled
                    and not (item.nfo.get("tmdbid") if item.nfo else None)
                ):
                    title = item.title or ""
                    year = item.year
                    shows = self.pm.search_tv(title, year)
                    if shows:
                        tv = shows[0]
                        tvid = str(tv.get("id"))
                        item.nfo["tmdbid"] = tvid
                        # Fetch episode details for first episode number
                        if item.season is not None and item.episodes:
                            ep_num = item.episodes[0]
                            ep = self.pm.get_episode(tvid, item.season, ep_num)
                            if ep and ep.get("name"):
                                item.nfo["episode_title"] = ep.get("name")

                # Re-score with updated metadata
                ident = IdentificationResult(item=item, reasons=["Online metadata (cached)"])
                new_score = score_identification(ident)
                result["score"] = new_score
                updated.append(result)
            except Exception as e:  # pragma: no cover - UI thread
                self.signals.progress.emit(f"Discover error: {e}")

        self.signals.finished.emit(updated)


class MainWindow(QMainWindow):
    """Main window for Rosey."""

    def __init__(self) -> None:
        super().__init__()
        self.items: list[dict] = []
        self.movie_nodes: dict[str, QTreeWidgetItem] = {}
        self.show_nodes: dict[str, QTreeWidgetItem] = {}
        self._thread_pool = QThreadPool.globalInstance()

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
        self.btn_scan.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_DirOpenIcon))
        self.btn_scan.clicked.connect(self.on_scan)
        toolbar.addWidget(self.btn_scan)

        self.btn_settings = QPushButton("Settings")
        self.btn_settings.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_VistaShield))
        self.btn_settings.clicked.connect(self.on_settings)
        toolbar.addWidget(self.btn_settings)

        self.btn_select_green = QPushButton("Select All Green")
        self.btn_select_green.setIcon(
            self.style().standardIcon(QStyle.StandardPixmap.SP_DialogApplyButton)
        )
        self.btn_select_green.clicked.connect(self.on_select_green)
        toolbar.addWidget(self.btn_select_green)

        self.btn_move = QPushButton("Move Selected")
        self.btn_move.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_ArrowForward))
        self.btn_move.clicked.connect(self.on_move_selected)
        toolbar.addWidget(self.btn_move)

        self.btn_clear = QPushButton("Clear Selection")
        self.btn_clear.setIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_TrashIcon))
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
        self.tree_movies.setIcon(
            0, self.style().standardIcon(QStyle.StandardPixmap.SP_ComputerIcon)
        )
        self.tree_shows = QTreeWidgetItem(self.tree, ["Shows"])
        self.tree_shows.setIcon(0, self.style().standardIcon(QStyle.StandardPixmap.SP_DriveNetIcon))
        self.tree_unknown = QTreeWidgetItem(self.tree, ["Unknown"])
        self.tree_unknown.setIcon(
            0, self.style().standardIcon(QStyle.StandardPixmap.SP_MessageBoxQuestion)
        )

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
        # self.add_seed_data()

    def log_activity(self, message: str) -> None:
        """Log a message to the activity log, prepending at the top."""
        cursor = self.activity_log.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.Start)
        cursor.insertText(message + "\n")

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

    def update_tree(self) -> None:
        """Update tree with discovered movies and shows."""
        # Clear children
        self.tree_movies.takeChildren()
        self.tree_shows.takeChildren()
        self.tree_unknown.takeChildren()
        self.movie_nodes.clear()
        self.show_nodes.clear()

        for result in self.items:
            item = result["item"]
            if item.kind == "movie":
                title = item.title or "Unknown Movie"
                if title not in self.movie_nodes:
                    node = QTreeWidgetItem(self.tree_movies, [title])
                    self.movie_nodes[title] = node
            elif item.kind == "episode":
                title = item.title or "Unknown Show"
                if title not in self.show_nodes:
                    node = QTreeWidgetItem(self.tree_shows, [title])
                    self.show_nodes[title] = node

    @Slot()
    def on_scan(self) -> None:
        """Handle scan button click."""
        source = self.config.paths.source
        if not source:
            self.log_activity("No source path configured. Open Settings to set Source Folder.")
            self.statusBar().showMessage("No source path configured")
            return

        self.log_activity(f"Starting scan: {source}")
        self.statusBar().showMessage("Scanning...")
        self.btn_scan.setEnabled(False)

        worker = ScanWorker(
            source,
            max_workers=self.config.scanning.concurrency_local,
            follow_symlinks=self.config.scanning.follow_symlinks,
            movies_root=self.config.paths.movies,
            tv_root=self.config.paths.tv,
        )
        worker.signals.progress.connect(self.on_scan_progress)
        worker.signals.finished.connect(self.on_scan_finished)
        self._thread_pool.start(worker)

    @Slot(str)
    def on_scan_progress(self, message: str) -> None:
        """Receive progress updates from scan worker."""
        self.log_activity(message)

    @Slot(list)
    def on_scan_finished(self, items: list) -> None:
        """Handle scan completion and populate table."""
        self.items = items
        self.populate_table(items)
        self.update_tree()
        self.log_activity(f"Scan completed - {len(items)} items")
        self.statusBar().showMessage("Scan completed")
        self.btn_scan.setEnabled(True)

    def _get_selected_rows(self) -> list[int]:
        rows: list[int] = []
        for row in range(self.table.rowCount()):
            check_item = self.table.item(row, 0)
            if check_item and check_item.checkState() == Qt.CheckState.Checked:
                rows.append(row)
        return rows

    def _items_from_rows(self, rows: list[int]) -> list[dict]:
        return [self.items[row] for row in rows if 0 <= row < len(self.items)]

    @Slot()
    def on_move_selected(self) -> None:
        """Move selected items (respects dry-run setting)."""
        rows = self._get_selected_rows()
        if not rows:
            self.log_activity("No items selected to move")
            self.statusBar().showMessage("No items selected")
            return

        to_move = self._items_from_rows(rows)

        dry_run = self.config.behavior.dry_run
        default_policy = self.config.behavior.conflict_policy

        # Resolve conflicts upfront and ensure policy matches mover expectations
        # mover accepts only: "skip", "replace", "keep_both"
        move_specs: list[dict] = []
        for r in to_move:
            item = r["item"]
            dest = r["destination"]

            eff_policy: MoverPolicy
            if default_policy == "skip":
                eff_policy = "skip"
            elif default_policy == "replace":
                eff_policy = "replace"
            elif default_policy == "keep_both":
                eff_policy = "keep_both"
            else:  # "ask"
                if Path(dest).exists():
                    from rosey.ui.conflict_dialog import ConflictDialog

                    dlg = ConflictDialog(item.source_path, dest, self)
                    chosen = dlg.get_policy() if dlg.exec() else "skip"
                    if chosen in ("skip", "replace", "keep_both"):
                        eff_policy = cast(MoverPolicy, chosen)
                    else:
                        eff_policy = "skip"
                else:
                    # No conflict expected; default to safe policy
                    eff_policy = "skip"

            move_specs.append({"item": item, "destination": dest, "policy": eff_policy})

        progress = ProgressDialog("Moving Files", self)
        cancelled = {"flag": False}

        def on_cancel() -> None:
            cancelled["flag"] = True

        progress.cancel_requested.connect(on_cancel)
        progress.show()
        self.statusBar().showMessage("Moving...")

        class MoveWorker(QRunnable):
            class Signals(QWidget):
                progress = Signal(str)
                finished = Signal(dict)

            def __init__(self, items: list[dict]) -> None:
                super().__init__()
                self.items = items
                self.signals = self.Signals()

            def run(self) -> None:  # noqa: D401
                moved = 0
                errors: list[str] = []
                total = len(self.items)
                for idx, spec in enumerate(self.items, start=1):
                    if cancelled["flag"]:
                        self.signals.progress.emit("Operation cancelled by user")
                        break
                    item = spec["item"]
                    dest = spec["destination"]
                    policy = spec.get("policy", "skip")
                    self.signals.progress.emit(f"Moving {idx}/{total}: {dest}")
                    try:
                        move_result = move_with_sidecars(
                            item,
                            dest,
                            conflict_policy=policy,
                            dry_run=dry_run,
                        )
                        if move_result.success:
                            moved += 1
                        else:
                            errors.extend(move_result.errors)
                    except Exception as e:  # pragma: no cover - UI thread
                        errors.append(str(e))

                    # Emit incremental progress
                    self.signals.progress.emit(f"Progress: {idx}/{total}")

                self.signals.finished.emit({"moved": moved, "total": total, "errors": errors})

        worker = MoveWorker(move_specs)
        worker.signals.progress.connect(progress.append_detail)
        worker.signals.progress.connect(progress.set_status)

        def on_finished(summary: dict) -> None:
            moved = summary.get("moved", 0)
            total = summary.get("total", 0)
            errors = summary.get("errors", [])
            for err in errors:
                progress.append_detail(f"Error: {err}")
            progress.set_complete(success=len(errors) == 0)
            self.statusBar().showMessage(f"Move complete: {moved}/{total}")

        worker.signals.finished.connect(on_finished)
        self._thread_pool.start(worker)

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

        self.log_activity(f"Selected {count} items with green confidence (>=70%)")
        self.statusBar().showMessage(f"Selected {count} green items")

    @Slot()
    def on_clear_selection(self) -> None:
        """Clear all selections."""
        for row in range(self.table.rowCount()):
            check_item = self.table.item(row, 0)
            if check_item:
                check_item.setCheckState(Qt.CheckState.Unchecked)

        self.log_activity("Cleared all selections")
        self.statusBar().showMessage("Selections cleared")

    def on_filter(self, filter_type: str) -> None:
        """Filter table by confidence level."""
        self.log_activity(f"Filtering: {filter_type}")

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

        item = selected[0]
        item_text = item.text(0)
        self.log_activity(f"Tree selection: {item_text}")

        # Check if it's a top level
        if item.parent() is None:
            # Top level
            if item == self.tree_movies:
                filter_type = "movie"
            elif item == self.tree_shows:
                filter_type = "episode"
            else:
                filter_type = "unknown"
            # Filter by type
            for row in range(self.table.rowCount()):
                type_item = self.table.item(row, 1)
                if type_item:
                    item_type = type_item.text()
                    hide = (
                        (filter_type == "movie" and item_type != "MOVIE")
                        or (filter_type == "episode" and item_type != "EPISODE")
                        or (filter_type == "unknown" and item_type != "UNKNOWN")
                    )
                    self.table.setRowHidden(row, hide)
        else:
            # Child node
            parent = item.parent()
            if parent == self.tree_movies:
                # Filter by movie title
                selected_title = item.text(0)
                for row in range(self.table.rowCount()):
                    type_item = self.table.item(row, 1)
                    name_item = self.table.item(row, 2)
                    if type_item and name_item:
                        item_type = type_item.text()
                        item_name = name_item.text()
                        hide = item_type != "MOVIE" or not item_name.startswith(selected_title)
                        self.table.setRowHidden(row, hide)
            elif parent == self.tree_shows:
                selected_title = item.text(0)
                for row in range(self.table.rowCount()):
                    type_item = self.table.item(row, 1)
                    name_item = self.table.item(row, 2)
                    if type_item and name_item:
                        item_type = type_item.text()
                        item_name = name_item.text()
                        hide = item_type != "EPISODE" or not item_name.startswith(selected_title)
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

            self.log_activity("Settings saved")
            self.statusBar().showMessage("Settings updated")

    @Slot()
    def on_tree_context_menu(self, position) -> None:  # type: ignore[no-untyped-def]
        """Handle tree context menu request."""
        item = self.tree.itemAt(position)
        if not item:
            return

        menu = QMenu(self)
        identify_action = menu.addAction("Identify...")

        action = menu.exec(self.tree.viewport().mapToGlobal(position))

        if action == identify_action:
            self.on_identify(item)

    def on_identify(self, tree_item: QTreeWidgetItem) -> None:
        """Handle identify action using manual dialog."""
        # Determine initial values from tree item
        initial_title = ""
        initial_type = "movie"

        if tree_item.parent() == self.tree_movies:
            initial_type = "movie"
            initial_title = tree_item.text(0)
        elif tree_item.parent() == self.tree_shows:
            initial_type = "episode"
            initial_title = tree_item.text(0)
        # For top-level items, use defaults

        dialog = IdentifyDialog(self.provider_manager, initial_title, initial_type, self)
        if dialog.exec():
            result = dialog.get_result()
            if result:
                # Update matching items
                target_title = initial_title or result.get("title", "")
                target_type = initial_type

                for item_dict in self.items:
                    item = item_dict["item"]
                    if item.title == target_title and (
                        (target_type == "movie" and item.kind == "movie")
                        or (target_type == "episode" and item.kind == "episode")
                    ):
                        # Update from result
                        if "id" in result:  # From provider
                            item.nfo = item.nfo or {}
                            item.nfo["tmdbid"] = str(result["id"])
                            item.title = result.get("title", result.get("name", item.title))
                            date = result.get("release_date", result.get("first_air_date", ""))
                            if date:
                                item.year = int(date[:4])
                        else:  # Manual
                            item.title = result.get("title", item.title)
                            if result.get("year"):
                                with contextlib.suppress(ValueError):
                                    item.year = int(result["year"])
                            if result.get("season") and item.kind == "episode":
                                with contextlib.suppress(ValueError):
                                    item.season = int(result["season"])
                            if result.get("episode") and item.kind == "episode":
                                with contextlib.suppress(ValueError):
                                    item.episodes = [int(result["episode"])]

                        # Re-score
                        from rosey.models import IdentificationResult

                        ident = IdentificationResult(item=item, reasons=["Manual identification"])
                        new_score = score_identification(ident)
                        item_dict["score"] = new_score

                        # Re-plan destination
                        new_dest = plan_path(
                            item,
                            movies_root=self.config.paths.movies,
                            tv_root=self.config.paths.tv,
                        )
                        item_dict["destination"] = new_dest

                # Update UI
                self.populate_table(self.items)
                self.update_tree()
                self.log_activity("Manual identification completed")


def main() -> int:
    """Run the UI."""
    app = QApplication([])
    window = MainWindow()
    window.show()
    return app.exec()


if __name__ == "__main__":
    main()
