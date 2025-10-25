"""Logging utilities with rotation and redaction."""

import logging
import os
import re
from logging.handlers import RotatingFileHandler
from pathlib import Path


def setup_logging(
    log_file: str = "",
    level: str = "INFO",
    max_bytes: int = 10 * 1024 * 1024,  # 10MB
    backup_count: int = 5,
    redact_secrets: bool = True,
) -> None:
    """
    Configure logging with rotation and optional redaction.

    Args:
        log_file: Path to log file. If empty, logs to console only.
        level: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
        max_bytes: Maximum size of log file before rotation
        backup_count: Number of backup files to keep
        redact_secrets: Whether to redact sensitive information
    """
    # Clear existing handlers
    root_logger = logging.getLogger()
    root_logger.setLevel(getattr(logging, level.upper(), logging.INFO))

    # Clear existing handlers
    for handler in root_logger.handlers[:]:
        root_logger.removeHandler(handler)

    # Create formatter
    formatter = logging.Formatter(
        "%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
    )

    # Add file handler if log_file provided
    if log_file:
        log_path = Path(log_file)
        log_path.parent.mkdir(parents=True, exist_ok=True)

        file_handler = RotatingFileHandler(
            log_path,
            maxBytes=max_bytes,
            backupCount=backup_count,
            encoding="utf-8",
        )
        file_handler.setFormatter(formatter)

        if redact_secrets:
            file_handler.addFilter(RedactingFilter())

        root_logger.addHandler(file_handler)

    # Console handler
    console_handler = logging.StreamHandler()
    console_handler.setFormatter(formatter)

    if redact_secrets:
        console_handler.addFilter(RedactingFilter())

    root_logger.addHandler(console_handler)


class RedactingFilter(logging.Filter):
    """Filter to redact sensitive information from log messages."""

    def __init__(self) -> None:
        super().__init__()
        self.patterns = [
            # API keys (various formats)
            (
                re.compile(r"(api[_-]?key[=:\s]+)['\"]?[\w\-]{16,}['\"]?", re.IGNORECASE),
                r"\1***REDACTED***",
            ),
            (
                re.compile(r"(token[=:\s]+)['\"]?[\w\-]{16,}['\"]?", re.IGNORECASE),
                r"\1***REDACTED***",
            ),
            (re.compile(r"(bearer\s+)['\"]?[\w\-]{16,}['\"]?", re.IGNORECASE), r"\1***REDACTED***"),
            # Passwords
            (
                re.compile(r"(password[=:\s]+)['\"]?[^'\"\s]{3,}['\"]?", re.IGNORECASE),
                r"\1***REDACTED***",
            ),
            (
                re.compile(r"(passwd[=:\s]+)['\"]?[^'\"\s]{3,}['\"]?", re.IGNORECASE),
                r"\1***REDACTED***",
            ),
            (
                re.compile(r"(pwd[=:\s]+)['\"]?[^'\"\s]{3,}['\"]?", re.IGNORECASE),
                r"\1***REDACTED***",
            ),
            # URLs with credentials
            (re.compile(r"(https?://)[^:]+:[^@]+@", re.IGNORECASE), r"\1***REDACTED***@"),
        ]

    def filter(self, record: logging.LogRecord) -> bool:
        """Redact sensitive information from log record."""
        if hasattr(record, "msg") and isinstance(record.msg, str):
            for pattern, replacement in self.patterns:
                record.msg = pattern.sub(replacement, record.msg)

        # Also redact in args if they exist
        if hasattr(record, "args") and record.args:
            if isinstance(record.args, dict):
                record.args = {k: self._redact_value(v) for k, v in record.args.items()}
            elif isinstance(record.args, tuple | list):
                record.args = tuple(self._redact_value(v) for v in record.args)

        return True

    def _redact_value(self, value: object) -> object:
        """Redact a single value."""
        if isinstance(value, str):
            for pattern, replacement in self.patterns:
                value = pattern.sub(replacement, value)
        return value


def get_log_path() -> Path:
    """Get platform-specific log file path."""
    if os.name == "nt":  # Windows
        base = Path(os.environ.get("APPDATA", "~"))
    else:  # Linux/macOS
        base = Path(os.environ.get("XDG_DATA_HOME", "~/.local/share"))

    log_dir = base.expanduser() / "rosey" / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    return log_dir / "rosey.log"
