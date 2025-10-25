"""Rosey application entry point."""

import sys

from PySide6.QtWidgets import QApplication

from rosey.config import load_config


def main() -> int:
    """Main application entry point."""
    # Load configuration
    config = load_config()
    print(f"Rosey v0.1.0 - Config loaded from {config.version}")

    # Create Qt application
    app = QApplication(sys.argv)
    app.setApplicationName("Rosey")
    app.setOrganizationName("Rosey")

    # TODO: Create and show main window
    print("UI not yet implemented. Exiting.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
