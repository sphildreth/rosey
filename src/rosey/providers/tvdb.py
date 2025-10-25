"""TVDB provider implementation (optional)."""

import time
from typing import Any

import httpx

from rosey.providers.base import MetadataProvider


class TVDBProvider(MetadataProvider):
    """TVDB metadata provider with rate limiting."""

    BASE_URL = "https://api4.thetvdb.com/v4"
    MAX_RPS = 2  # Conservative rate limit

    def __init__(
        self,
        api_key: str,
        language: str = "eng",
        timeout: float = 10.0,
    ):
        """Initialize TVDB provider.

        Args:
            api_key: TVDB API key
            language: Language code (e.g., 'eng')
            timeout: Request timeout in seconds
        """
        self.api_key = api_key
        self.language = language
        self.timeout = timeout
        self._client = httpx.Client(timeout=timeout)
        self._token: str | None = None
        self._token_expires: float = 0

    def _ensure_token(self) -> bool:
        """Ensure we have a valid authentication token."""
        if self._token and time.time() < self._token_expires:
            return True

        try:
            response = self._client.post(
                f"{self.BASE_URL}/login",
                json={"apikey": self.api_key},
            )
            response.raise_for_status()
            data = response.json()
            self._token = data.get("data", {}).get("token")
            # TVDB tokens expire after 1 month, refresh before then
            self._token_expires = time.time() + (30 * 24 * 3600) - 3600
            return self._token is not None
        except (httpx.RequestError, httpx.HTTPStatusError):
            return False

    def _request(
        self, endpoint: str, params: dict[str, Any] | None = None
    ) -> dict[str, Any] | None:
        """Make request to TVDB API.

        Args:
            endpoint: API endpoint
            params: Query parameters

        Returns:
            JSON response or None on error
        """
        if not self._ensure_token():
            return None

        headers = {"Authorization": f"Bearer {self._token}"}
        url = f"{self.BASE_URL}{endpoint}"

        try:
            response = self._client.get(url, headers=headers, params=params)
            response.raise_for_status()
            return response.json()  # type: ignore[no-any-return]
        except (httpx.RequestError, httpx.HTTPStatusError):
            return None

    def search_movie(self, title: str, year: int | None = None) -> list[dict[str, Any]]:
        """Search for movies by title and optional year."""
        params: dict[str, Any] = {"query": title, "type": "movie"}
        if year:
            params["year"] = str(year)

        result = self._request("/search", params)
        if result and "data" in result:
            return result["data"]  # type: ignore[no-any-return]
        return []

    def search_tv(self, title: str, year: int | None = None) -> list[dict[str, Any]]:
        """Search for TV shows by title and optional year."""
        params: dict[str, Any] = {"query": title, "type": "series"}
        if year:
            params["year"] = str(year)

        result = self._request("/search", params)
        if result and "data" in result:
            return result["data"]  # type: ignore[no-any-return]
        return []

    def get_movie_by_id(self, movie_id: str) -> dict[str, Any] | None:
        """Get movie details by ID."""
        result = self._request(f"/movies/{movie_id}/extended")
        if result and "data" in result:
            return result["data"]  # type: ignore[no-any-return]
        return None

    def get_tv_by_id(self, tv_id: str) -> dict[str, Any] | None:
        """Get TV show details by ID."""
        result = self._request(f"/series/{tv_id}/extended")
        if result and "data" in result:
            return result["data"]  # type: ignore[no-any-return]
        return None

    def get_episode(self, tv_id: str, season: int, episode: int) -> dict[str, Any] | None:
        """Get episode details."""
        # TVDB requires fetching season data first
        result = self._request(f"/series/{tv_id}/episodes/default")
        if not result or "data" not in result:
            return None

        episodes: list[dict[str, Any]] = result["data"].get("episodes", [])
        for ep in episodes:
            if ep.get("seasonNumber") == season and ep.get("number") == episode:
                return ep

        return None

    def close(self) -> None:
        """Close the HTTP client."""
        self._client.close()
