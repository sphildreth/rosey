"""Settings dialog for Rosey configuration."""

from PySide6.QtWidgets import (
    QCheckBox,
    QDialog,
    QDialogButtonBox,
    QFormLayout,
    QGroupBox,
    QLabel,
    QLineEdit,
    QSpinBox,
    QTabWidget,
    QVBoxLayout,
    QWidget,
)

from rosey.config import RoseyConfig


class SettingsDialog(QDialog):
    """Settings dialog."""

    def __init__(self, config: RoseyConfig, parent: QWidget | None = None):
        """Initialize settings dialog.

        Args:
            config: Current configuration
            parent: Parent widget
        """
        super().__init__(parent)
        self.config = config
        self.setWindowTitle("Settings")
        self.setMinimumWidth(600)
        self.init_ui()

    def init_ui(self) -> None:
        """Initialize UI."""
        layout = QVBoxLayout(self)

        # Tab widget
        tabs = QTabWidget()

        # Paths tab
        paths_tab = self._create_paths_tab()
        tabs.addTab(paths_tab, "Paths")

        # Providers tab
        providers_tab = self._create_providers_tab()
        tabs.addTab(providers_tab, "Online Providers")

        # Behavior tab
        behavior_tab = self._create_behavior_tab()
        tabs.addTab(behavior_tab, "Behavior")

        # Scanning tab
        scanning_tab = self._create_scanning_tab()
        tabs.addTab(scanning_tab, "Scanning")

        layout.addWidget(tabs)

        # Buttons
        buttons = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Ok | QDialogButtonBox.StandardButton.Cancel
        )
        buttons.accepted.connect(self.accept)
        buttons.rejected.connect(self.reject)
        layout.addWidget(buttons)

    def _create_paths_tab(self) -> QWidget:
        """Create paths configuration tab."""
        widget = QWidget()
        layout = QFormLayout(widget)

        self.source_edit = QLineEdit(self.config.paths.source)
        layout.addRow("Source Folder:", self.source_edit)

        self.movies_edit = QLineEdit(self.config.paths.movies)
        layout.addRow("Movies Target:", self.movies_edit)

        self.tv_edit = QLineEdit(self.config.paths.tv)
        layout.addRow("TV Target:", self.tv_edit)

        return widget

    def _create_providers_tab(self) -> QWidget:
        """Create providers configuration tab."""
        widget = QWidget()
        layout = QVBoxLayout(widget)

        # Enable/disable checkbox
        self.providers_enabled = QCheckBox("Enable Online Metadata Lookups")
        self.providers_enabled.setChecked(self.config.identification.use_online_providers)
        layout.addWidget(self.providers_enabled)

        # Info label
        info = QLabel(
            "When enabled, Rosey will query TMDB/TVDB for metadata.\n"
            "Requires valid API keys. Lookups are cached and rate-limited."
        )
        info.setWordWrap(True)
        layout.addWidget(info)

        # TMDB group
        tmdb_group = QGroupBox("TMDB (The Movie Database)")
        tmdb_layout = QFormLayout(tmdb_group)

        self.tmdb_api_key = QLineEdit(self.config.providers.tmdb_api_key)
        self.tmdb_api_key.setEchoMode(QLineEdit.EchoMode.Password)
        tmdb_layout.addRow("API Key:", self.tmdb_api_key)

        self.tmdb_language = QLineEdit(self.config.providers.tmdb_language)
        tmdb_layout.addRow("Language:", self.tmdb_language)

        self.tmdb_region = QLineEdit(self.config.providers.tmdb_region)
        tmdb_layout.addRow("Region:", self.tmdb_region)

        layout.addWidget(tmdb_group)

        # TVDB group (optional)
        tvdb_group = QGroupBox("TVDB (TheTVDB) - Optional")
        tvdb_layout = QFormLayout(tvdb_group)

        self.tvdb_api_key = QLineEdit(self.config.providers.tvdb_api_key)
        self.tvdb_api_key.setEchoMode(QLineEdit.EchoMode.Password)
        tvdb_layout.addRow("API Key:", self.tvdb_api_key)

        self.tvdb_language = QLineEdit(self.config.providers.tvdb_language)
        tvdb_layout.addRow("Language:", self.tvdb_language)

        layout.addWidget(tvdb_group)

        # Cache settings
        cache_group = QGroupBox("Cache Settings")
        cache_layout = QFormLayout(cache_group)

        self.cache_ttl = QSpinBox()
        self.cache_ttl.setRange(1, 365)
        self.cache_ttl.setValue(self.config.providers.cache_ttl_days)
        self.cache_ttl.setSuffix(" days")
        cache_layout.addRow("Cache TTL:", self.cache_ttl)

        layout.addWidget(cache_group)

        layout.addStretch()

        return widget

    def _create_behavior_tab(self) -> QWidget:
        """Create behavior configuration tab."""
        widget = QWidget()
        layout = QFormLayout(widget)

        self.dry_run = QCheckBox("Dry Run Mode (Preview Only)")
        self.dry_run.setChecked(self.config.behavior.dry_run)
        layout.addRow("", self.dry_run)

        self.auto_select_green = QCheckBox("Auto-Select Green Items")
        self.auto_select_green.setChecked(self.config.behavior.auto_select_green)
        layout.addRow("", self.auto_select_green)

        return widget

    def _create_scanning_tab(self) -> QWidget:
        """Create scanning configuration tab."""
        widget = QWidget()
        layout = QFormLayout(widget)

        self.concurrency_local = QSpinBox()
        self.concurrency_local.setRange(1, 32)
        self.concurrency_local.setValue(self.config.scanning.concurrency_local)
        layout.addRow("Local Concurrency:", self.concurrency_local)

        self.concurrency_network = QSpinBox()
        self.concurrency_network.setRange(1, 16)
        self.concurrency_network.setValue(self.config.scanning.concurrency_network)
        layout.addRow("Network Concurrency:", self.concurrency_network)

        self.follow_symlinks = QCheckBox("Follow Symbolic Links")
        self.follow_symlinks.setChecked(self.config.scanning.follow_symlinks)
        layout.addRow("", self.follow_symlinks)

        return widget

    def get_config(self) -> RoseyConfig:
        """Get updated configuration from dialog.

        Returns:
            Updated configuration
        """
        # Update paths
        self.config.paths.source = self.source_edit.text()
        self.config.paths.movies = self.movies_edit.text()
        self.config.paths.tv = self.tv_edit.text()

        # Update providers
        self.config.identification.use_online_providers = self.providers_enabled.isChecked()
        self.config.providers.tmdb_api_key = self.tmdb_api_key.text()
        self.config.providers.tmdb_language = self.tmdb_language.text()
        self.config.providers.tmdb_region = self.tmdb_region.text()
        self.config.providers.tvdb_api_key = self.tvdb_api_key.text()
        self.config.providers.tvdb_language = self.tvdb_language.text()
        self.config.providers.cache_ttl_days = self.cache_ttl.value()

        # Update behavior
        self.config.behavior.dry_run = self.dry_run.isChecked()
        self.config.behavior.auto_select_green = self.auto_select_green.isChecked()

        # Update scanning
        self.config.scanning.concurrency_local = self.concurrency_local.value()
        self.config.scanning.concurrency_network = self.concurrency_network.value()
        self.config.scanning.follow_symlinks = self.follow_symlinks.isChecked()

        return self.config
