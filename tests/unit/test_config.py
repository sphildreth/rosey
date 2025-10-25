"""Tests for configuration management."""

from pathlib import Path

import pytest

from rosey.config import RoseyConfig, load_config, save_config


def test_default_config() -> None:
    """Test default configuration values."""
    config = RoseyConfig()

    assert config.version == "1.0"
    assert config.behavior.dry_run is True
    assert config.ui.theme == "system"
    assert config.scanning.concurrency_local == 8
    assert config.identification.use_online_providers is False


def test_config_serialization() -> None:
    """Test config can be serialized and deserialized."""
    config = RoseyConfig()
    config.paths.source = "/test/source"
    config.paths.movies = "/test/movies"
    config.ui.theme = "dark"

    # Serialize
    data = config.model_dump()

    # Deserialize
    config2 = RoseyConfig(**data)

    assert config2.paths.source == "/test/source"
    assert config2.paths.movies == "/test/movies"
    assert config2.ui.theme == "dark"


def test_save_and_load_config(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test saving and loading config from disk."""
    # Override config path to temp directory
    config_file = tmp_path / "rosey.json"

    def mock_get_config_path() -> Path:
        return config_file

    monkeypatch.setattr("rosey.config.get_config_path", mock_get_config_path)

    # Create and save config
    config = RoseyConfig()
    config.paths.source = "/my/source"
    config.behavior.dry_run = False

    save_config(config)

    # Verify file exists
    assert config_file.exists()

    # Load config
    loaded = load_config()

    assert loaded.paths.source == "/my/source"
    assert loaded.behavior.dry_run is False


def test_load_missing_config(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test loading config when file doesn't exist returns defaults."""
    config_file = tmp_path / "nonexistent.json"

    def mock_get_config_path() -> Path:
        return config_file

    monkeypatch.setattr("rosey.config.get_config_path", mock_get_config_path)

    config = load_config()

    # Should return defaults
    assert config.version == "1.0"
    assert config.behavior.dry_run is True


def test_load_invalid_config(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test loading invalid config returns defaults."""
    config_file = tmp_path / "invalid.json"
    config_file.write_text("{ invalid json }")

    def mock_get_config_path() -> Path:
        return config_file

    monkeypatch.setattr("rosey.config.get_config_path", mock_get_config_path)

    config = load_config()

    # Should return defaults on error
    assert config.version == "1.0"
