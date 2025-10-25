"""TMDB provider implementation."""

import time
from collections import deque
from threading import Lock
from typing import Any

import httpx

from rosey.providers.base import MetadataProvider


class TMDBProvider(MetadataProvider):
    """TMDB metadata provider with rate limiting."""

    BASE_URL = "https://api.themoviedb.org/3"
    MAX_RPS = 4  # TMDB allows ~40 requests/10s, we'll be conservative

    def __init__(
        self,
        api_key: str,
        language: str = "en-US",
        region: str = "US",
        timeout: float = 10.0,
    ):
        """Initialize TMDB provider.

        Args:
            api_key: TMDB API key
            language: Language code (e.g., 'en-US')
            region: Region code (e.g., 'US')
            timeout: Request timeout in seconds
        """
        self.api_key = api_key
        self.language = language
        self.region = region
        self.timeout = timeout
        self._client = httpx.Client(timeout=timeout)
        self._rate_limiter = RateLimiter(max_rps=self.MAX_RPS)

    def _request(
        self, endpoint: str, params: dict[str, Any] | None = None
    ) -> dict[str, Any] | None:
        """Make rate-limited request to TMDB API.

        Args:
            endpoint: API endpoint (e.g., '/search/movie')
            params: Query parameters

        Returns:
            JSON response or None on error
        """
        self._rate_limiter.acquire()

        if params is None:
            params = {}

        params.setdefault("api_key", self.api_key)
        params.setdefault("language", self.language)

        url = f"{self.BASE_URL}{endpoint}"

        try:
            response = self._client.get(url, params=params)
            response.raise_for_status()
            return response.json()  # type: ignore[no-any-return]
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 429:
                # Rate limited, apply backoff
                retry_after = int(e.response.headers.get("Retry-After", 2))
                time.sleep(retry_after)
                return self._request(endpoint, params)
            # Other HTTP errors
            return None
        except (httpx.RequestError, httpx.TimeoutException):
            return None

    def search_movie(self, title: str, year: int | None = None) -> list[dict[str, Any]]:
        """Search for movies by title and optional year."""
        params: dict[str, Any] = {"query": title, "region": self.region}
        if year:
            params["year"] = str(year)

        result = self._request("/search/movie", params)
        if result and "results" in result:
            return result["results"]  # type: ignore[no-any-return]
        return []

    def search_tv(self, title: str, year: int | None = None) -> list[dict[str, Any]]:
        """Search for TV shows by title and optional year."""
        params: dict[str, Any] = {"query": title}
        if year:
            params["first_air_date_year"] = str(year)

        result = self._request("/search/tv", params)
        if result and "results" in result:
            return result["results"]  # type: ignore[no-any-return]
        return []

    def get_movie_by_id(self, movie_id: str) -> dict[str, Any] | None:
        """Get movie details by ID."""
        return self._request(f"/movie/{movie_id}")

    def get_tv_by_id(self, tv_id: str) -> dict[str, Any] | None:
        """Get TV show details by ID."""
        return self._request(f"/tv/{tv_id}")

    def get_episode(self, tv_id: str, season: int, episode: int) -> dict[str, Any] | None:
        """Get episode details."""
        return self._request(f"/tv/{tv_id}/season/{season}/episode/{episode}")

    def close(self) -> None:
        """Close the HTTP client."""
        self._client.close()


class RateLimiter:
    """Token bucket rate limiter."""

    def __init__(self, max_rps: int):
        """Initialize rate limiter.

        Args:
            max_rps: Maximum requests per second
        """
        self.max_rps = max_rps
        self.interval = 1.0 / max_rps
        self.last_request_times: deque[float] = deque(maxlen=max_rps)
        self.lock = Lock()

    def acquire(self) -> None:
        """Acquire a token, blocking if necessary."""
        with self.lock:
            now = time.time()

            # Remove timestamps older than 1 second
            while self.last_request_times and now - self.last_request_times[0] >= 1.0:
                self.last_request_times.popleft()

            # If we've hit the limit, wait
            if len(self.last_request_times) >= self.max_rps:
                oldest = self.last_request_times[0]
                sleep_time = 1.0 - (now - oldest)
                if sleep_time > 0:
                    time.sleep(sleep_time)
                self.last_request_times.popleft()

            self.last_request_times.append(time.time())
