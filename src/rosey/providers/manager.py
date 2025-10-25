"""Provider manager with caching and graceful degradation."""

from pathlib import Path
from typing import Any

from rosey.providers.cache import ProviderCache
from rosey.providers.tmdb import TMDBProvider
from rosey.providers.tvdb import TVDBProvider


class ProviderManager:
    """Manages metadata providers with caching."""

    def __init__(
        self,
        cache_dir: Path,
        cache_ttl_days: int = 30,
        enabled: bool = False,
    ):
        """Initialize provider manager.

        Args:
            cache_dir: Directory for cache
            cache_ttl_days: Cache TTL in days
            enabled: Whether online lookups are enabled
        """
        self.cache = ProviderCache(cache_dir, cache_ttl_days)
        self.enabled = enabled
        self._tmdb: TMDBProvider | None = None
        self._tvdb: TVDBProvider | None = None

    def configure_tmdb(self, api_key: str, language: str = "en-US", region: str = "US") -> None:
        """Configure TMDB provider.

        Args:
            api_key: TMDB API key
            language: Language code
            region: Region code
        """
        if self._tmdb:
            self._tmdb.close()
        self._tmdb = TMDBProvider(api_key, language, region)

    def configure_tvdb(self, api_key: str, language: str = "eng") -> None:
        """Configure TVDB provider.

        Args:
            api_key: TVDB API key
            language: Language code
        """
        if self._tvdb:
            self._tvdb.close()
        self._tvdb = TVDBProvider(api_key, language)

    def search_movie(
        self, title: str, year: int | None = None, use_cache: bool = True
    ) -> list[dict[str, Any]]:
        """Search for movies.

        Args:
            title: Movie title
            year: Optional year
            use_cache: Whether to use cache

        Returns:
            List of movie results (empty if providers disabled/failed)
        """
        if not self.enabled or not self._tmdb:
            return []

        cache_key = f"{title}_{year or 'none'}"

        # Check cache
        if use_cache:
            cached = self.cache.get("tmdb", "search_movie", cache_key)
            if cached is not None:
                return cached if isinstance(cached, list) else []

        # Query provider
        try:
            results = self._tmdb.search_movie(title, year)
            if use_cache:
                self.cache.set("tmdb", "search_movie", cache_key, results)
            return results
        except Exception:
            # Graceful degradation
            return []

    def search_tv(
        self, title: str, year: int | None = None, use_cache: bool = True
    ) -> list[dict[str, Any]]:
        """Search for TV shows.

        Args:
            title: Show title
            year: Optional year
            use_cache: Whether to use cache

        Returns:
            List of TV show results (empty if providers disabled/failed)
        """
        if not self.enabled or not self._tmdb:
            return []

        cache_key = f"{title}_{year or 'none'}"

        # Check cache
        if use_cache:
            cached = self.cache.get("tmdb", "search_tv", cache_key)
            if cached is not None:
                return cached if isinstance(cached, list) else []

        # Query provider
        try:
            results = self._tmdb.search_tv(title, year)
            if use_cache:
                self.cache.set("tmdb", "search_tv", cache_key, results)
            return results
        except Exception:
            # Graceful degradation
            return []

    def get_movie_by_id(self, movie_id: str, use_cache: bool = True) -> dict[str, Any] | None:
        """Get movie details by ID.

        Args:
            movie_id: Movie ID
            use_cache: Whether to use cache

        Returns:
            Movie details or None
        """
        if not self.enabled or not self._tmdb:
            return None

        # Check cache
        if use_cache:
            cached = self.cache.get("tmdb", "movie", movie_id)
            if cached is not None:
                return cached if isinstance(cached, dict) else None

        # Query provider
        try:
            result = self._tmdb.get_movie_by_id(movie_id)
            if result and use_cache:
                self.cache.set("tmdb", "movie", movie_id, result)
            return result
        except Exception:
            return None

    def get_tv_by_id(self, tv_id: str, use_cache: bool = True) -> dict[str, Any] | None:
        """Get TV show details by ID.

        Args:
            tv_id: TV show ID
            use_cache: Whether to use cache

        Returns:
            TV show details or None
        """
        if not self.enabled or not self._tmdb:
            return None

        # Check cache
        if use_cache:
            cached = self.cache.get("tmdb", "tv", tv_id)
            if cached is not None:
                return cached if isinstance(cached, dict) else None

        # Query provider
        try:
            result = self._tmdb.get_tv_by_id(tv_id)
            if result and use_cache:
                self.cache.set("tmdb", "tv", tv_id, result)
            return result
        except Exception:
            return None

    def get_episode(
        self, tv_id: str, season: int, episode: int, use_cache: bool = True
    ) -> dict[str, Any] | None:
        """Get episode details.

        Args:
            tv_id: TV show ID
            season: Season number
            episode: Episode number
            use_cache: Whether to use cache

        Returns:
            Episode details or None
        """
        if not self.enabled or not self._tmdb:
            return None

        cache_key = f"{tv_id}_s{season}e{episode}"

        # Check cache
        if use_cache:
            cached = self.cache.get("tmdb", "episode", cache_key)
            if cached is not None:
                return cached if isinstance(cached, dict) else None

        # Query provider
        try:
            result = self._tmdb.get_episode(tv_id, season, episode)
            if result and use_cache:
                self.cache.set("tmdb", "episode", cache_key, result)
            return result
        except Exception:
            return None

    def close(self) -> None:
        """Close all provider connections."""
        if self._tmdb:
            self._tmdb.close()
        if self._tvdb:
            self._tvdb.close()
