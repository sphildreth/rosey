"""Rosey application entry point."""

import sys

from PySide6.QtWidgets import QApplication

from rosey.ui import MainWindow


def main() -> int:
    """Main application entry point."""
    # Create Qt application
    app = QApplication(sys.argv)
    app.setApplicationName("Rosey")
    app.setOrganizationName("Rosey")

    # Create and show main window
    window = MainWindow()
    window.show()

    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
