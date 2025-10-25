"""Base provider interface for online metadata lookups."""

from abc import ABC, abstractmethod
from typing import Any


class MetadataProvider(ABC):
    """Abstract base for metadata providers (TMDB, TVDB)."""

    @abstractmethod
    def search_movie(self, title: str, year: int | None = None) -> list[dict[str, Any]]:
        """Search for movies by title and optional year."""
        ...

    @abstractmethod
    def search_tv(self, title: str, year: int | None = None) -> list[dict[str, Any]]:
        """Search for TV shows by title and optional year."""
        ...

    @abstractmethod
    def get_movie_by_id(self, movie_id: str) -> dict[str, Any] | None:
        """Get movie details by ID."""
        ...

    @abstractmethod
    def get_tv_by_id(self, tv_id: str) -> dict[str, Any] | None:
        """Get TV show details by ID."""
        ...

    @abstractmethod
    def get_episode(self, tv_id: str, season: int, episode: int) -> dict[str, Any] | None:
        """Get episode details."""
        ...
