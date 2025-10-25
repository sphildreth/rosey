"""Tests for logging utilities."""

import logging
from pathlib import Path

from rosey.utils.logging import RedactingFilter, get_log_path, setup_logging


def test_setup_logging_creates_file(tmp_path):
    """Test that setup_logging creates log file."""
    log_file = tmp_path / "test.log"

    setup_logging(str(log_file), level="INFO")

    logger = logging.getLogger("test")
    logger.info("Test message")

    assert log_file.exists()
    content = log_file.read_text()
    assert "Test message" in content


def test_setup_logging_respects_level(tmp_path):
    """Test logging level filtering."""
    log_file = tmp_path / "test.log"

    setup_logging(str(log_file), level="WARNING")

    logger = logging.getLogger("test")
    logger.info("Info message")
    logger.warning("Warning message")

    content = log_file.read_text()
    assert "Info message" not in content
    assert "Warning message" in content


def test_setup_logging_rotation(tmp_path):
    """Test log rotation works."""
    log_file = tmp_path / "test.log"

    # Small max size to trigger rotation
    setup_logging(str(log_file), level="INFO", max_bytes=100, backup_count=2)

    logger = logging.getLogger("test")
    for i in range(50):
        logger.info(f"Message {i}")

    # Should have created backup files
    backups = list(tmp_path.glob("test.log.*"))
    assert len(backups) > 0


def test_redacting_filter_api_keys():
    """Test that API keys are redacted."""
    filter = RedactingFilter()

    record = logging.LogRecord(
        name="test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="api_key=abc123def456ghi789",
        args=(),
        exc_info=None,
    )

    filter.filter(record)

    assert "abc123def456ghi789" not in record.msg
    assert "***REDACTED***" in record.msg


def test_redacting_filter_tokens():
    """Test that tokens are redacted."""
    filter = RedactingFilter()

    record = logging.LogRecord(
        name="test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="Bearer token: eyJhbGciOiJIUzI1NiIsInR5cCI",
        args=(),
        exc_info=None,
    )

    filter.filter(record)

    assert "eyJhbGciOiJIUzI1NiIsInR5cCI" not in record.msg
    assert "***REDACTED***" in record.msg


def test_redacting_filter_passwords():
    """Test that passwords are redacted."""
    filter = RedactingFilter()

    record = logging.LogRecord(
        name="test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="Password: secret123",
        args=(),
        exc_info=None,
    )

    filter.filter(record)

    assert "secret123" not in record.msg
    assert "***REDACTED***" in record.msg


def test_redacting_filter_urls_with_credentials():
    """Test that URLs with credentials are redacted."""
    filter = RedactingFilter()

    record = logging.LogRecord(
        name="test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="Connecting to https://user:pass@example.com/api",
        args=(),
        exc_info=None,
    )

    filter.filter(record)

    assert "user:pass" not in record.msg
    assert "***REDACTED***@" in record.msg


def test_redacting_filter_preserves_safe_content():
    """Test that non-sensitive content is preserved."""
    filter = RedactingFilter()

    safe_msg = "Processing file /path/to/movie.mkv with confidence 85"
    record = logging.LogRecord(
        name="test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg=safe_msg,
        args=(),
        exc_info=None,
    )

    filter.filter(record)

    assert record.msg == safe_msg


def test_get_log_path_returns_valid_path():
    """Test that get_log_path returns a valid path."""
    path = get_log_path()

    assert isinstance(path, Path)
    assert path.name == "rosey.log"
    assert "rosey" in str(path)


def test_get_log_path_creates_directory():
    """Test that get_log_path creates parent directory."""
    path = get_log_path()

    assert path.parent.exists()


def test_setup_logging_with_redaction(tmp_path):
    """Test logging with redaction enabled."""
    log_file = tmp_path / "test.log"

    setup_logging(str(log_file), level="INFO", redact_secrets=True)

    logger = logging.getLogger("test")
    logger.info("API_KEY=super_secret_key_12345678")

    content = log_file.read_text()
    assert "super_secret_key_12345678" not in content
    assert "***REDACTED***" in content


def test_setup_logging_without_redaction(tmp_path):
    """Test logging with redaction disabled."""
    log_file = tmp_path / "test.log"

    setup_logging(str(log_file), level="INFO", redact_secrets=False)

    logger = logging.getLogger("test")
    logger.info("API_KEY=super_secret_key_12345678")

    content = log_file.read_text()
    assert "super_secret_key_12345678" in content
