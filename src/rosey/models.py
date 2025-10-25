"""Data models for Rosey."""

from pydantic import BaseModel, Field


class MediaItem(BaseModel):
    """A media item identified from filesystem."""

    kind: str  # "movie" | "show" | "episode" | "unknown"
    source_path: str
    title: str | None = None
    year: int | None = None
    season: int | None = None
    episodes: list[int] | None = None  # e.g., [1] or [1,2] for multi-episode
    part: int | None = None  # e.g., 1, 2 for multipart
    date: str | None = None  # YYYY-MM-DD for daily shows
    sidecars: list[str] = Field(default_factory=list)
    nfo: dict[str, str | None] = Field(default_factory=dict)  # ids/title/year/episode_title


class IdentificationResult(BaseModel):
    """Result of identifying a media item."""

    item: MediaItem
    reasons: list[str]
    online_metadata: dict | None = None
    errors: list[str] = Field(default_factory=list)


class Score(BaseModel):
    """Confidence score for an identification."""

    confidence: int  # 0–100
    reasons: list[str]


class MovePlan(BaseModel):
    """Plan for moving files."""

    destination_paths: list[str]
    conflicts: list[dict] = Field(default_factory=list)
    preflight: dict[str, bool] = Field(
        default_factory=lambda: {"free_space_ok": True, "perms_ok": True, "path_len_ok": True}
    )
    dry_run: bool = True


def _default_move_details() -> dict[str, list[str]]:
    """Create default move details dictionary."""
    return {"moved": [], "skipped": [], "replaced": [], "kept_both": []}


class MoveResult(BaseModel):
    """Result of a move operation."""

    success: bool
    details: dict[str, list[str]] = Field(default_factory=_default_move_details)
    rollback_performed: bool = False
    errors: list[str] = Field(default_factory=list)
