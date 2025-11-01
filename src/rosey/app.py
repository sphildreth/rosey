"""Rosey application entry point."""

import sys
from pathlib import Path

from PySide6.QtGui import QIcon
from PySide6.QtWidgets import QApplication

from rosey.ui import MainWindow


def main() -> int:
    """Main application entry point."""
    # Create Qt application
    app = QApplication(sys.argv)
    app.setApplicationName("Rosey")
    app.setOrganizationName("Rosey")

    # Set application icon
    icon_path = Path(__file__).parent.parent.parent / "graphics" / "rosey_64.png"
    if icon_path.exists():
        app.setWindowIcon(QIcon(str(icon_path)))

    # Create and show main window
    window = MainWindow()
    window.show()

    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
