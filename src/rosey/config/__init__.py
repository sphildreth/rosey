"""Configuration management for Rosey."""

import json
import os
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, Field


class PathsConfig(BaseModel):
    """Paths configuration."""

    source: str = ""
    movies: str = ""
    tv: str = ""


class WindowConfig(BaseModel):
    """Window state configuration."""

    width: int = 1200
    height: int = 800
    maximized: bool = False


class SplittersConfig(BaseModel):
    """Splitter positions."""

    main: list[int] = Field(default_factory=lambda: [300, 900])
    vertical: list[int] = Field(default_factory=lambda: [600, 200])


class UIConfig(BaseModel):
    """UI configuration."""

    theme: Literal["system", "light", "dark"] = "system"
    window: WindowConfig = Field(default_factory=WindowConfig)
    splitters: SplittersConfig = Field(default_factory=SplittersConfig)


class BehaviorConfig(BaseModel):
    """Behavior configuration."""

    dry_run: bool = True
    auto_select_green: bool = True
    conflict_policy: Literal["ask", "skip", "replace", "keep_both"] = "ask"


class ScanningConfig(BaseModel):
    """Scanning configuration."""

    concurrency_local: int = 8
    concurrency_network: int = 2
    follow_symlinks: bool = False


class IdentificationConfig(BaseModel):
    """Identification configuration."""

    use_online_providers: bool = False
    confidence_thresholds: dict[str, int] = Field(
        default_factory=lambda: {"green": 70, "yellow": 40}
    )
    prefer_nfo_ids: bool = True


class LoggingConfig(BaseModel):
    """Logging configuration."""

    level: Literal["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"] = "INFO"
    file_path: str = ""
    max_file_size_mb: int = 10
    backup_count: int = 5
    redact_secrets: bool = True
    log_to_console: bool = False


class RoseyConfig(BaseModel):
    """Main Rosey configuration."""

    version: str = "1.0"
    paths: PathsConfig = Field(default_factory=PathsConfig)
    ui: UIConfig = Field(default_factory=UIConfig)
    behavior: BehaviorConfig = Field(default_factory=BehaviorConfig)
    scanning: ScanningConfig = Field(default_factory=ScanningConfig)
    identification: IdentificationConfig = Field(default_factory=IdentificationConfig)
    logging: LoggingConfig = Field(default_factory=LoggingConfig)


def get_config_path() -> Path:
    """Get platform-specific config file path."""
    if os.name == "nt":  # Windows
        base = Path(os.environ.get("APPDATA", "~"))
    else:  # Linux/macOS
        base = Path(os.environ.get("XDG_CONFIG_HOME", "~/.config"))

    config_dir = base.expanduser() / "rosey"
    config_dir.mkdir(parents=True, exist_ok=True)
    return config_dir / "rosey.json"


def load_config() -> RoseyConfig:
    """Load configuration from disk or return defaults."""
    config_path = get_config_path()

    if not config_path.exists():
        return RoseyConfig()

    try:
        with open(config_path, encoding="utf-8") as f:
            data = json.load(f)
        return RoseyConfig(**data)
    except Exception as e:
        print(f"Warning: Failed to load config from {config_path}: {e}")
        print("Using default configuration.")
        return RoseyConfig()


def save_config(config: RoseyConfig) -> None:
    """Save configuration to disk."""
    config_path = get_config_path()

    try:
        with open(config_path, "w", encoding="utf-8") as f:
            json.dump(config.model_dump(), f, indent=2)
    except Exception as e:
        print(f"Error: Failed to save config to {config_path}: {e}")
