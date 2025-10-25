"""Tests for provider cache."""

import tempfile
import time
from pathlib import Path

from rosey.providers.cache import ProviderCache


def test_cache_init():
    """Test cache initialization."""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache = ProviderCache(Path(tmpdir), ttl_days=7)
        assert cache.db_path.exists()
        assert cache.ttl_seconds == 7 * 24 * 3600


def test_cache_set_and_get():
    """Test setting and getting cached data."""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache = ProviderCache(Path(tmpdir), ttl_days=7)

        data = {"id": 123, "title": "Test Movie"}
        cache.set("tmdb", "movie", "123", data)

        retrieved = cache.get("tmdb", "movie", "123")
        assert retrieved == data


def test_cache_get_nonexistent():
    """Test getting non-existent cache entry."""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache = ProviderCache(Path(tmpdir), ttl_days=7)

        result = cache.get("tmdb", "movie", "999")
        assert result is None


def test_cache_expiration():
    """Test cache entry expiration."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Very short TTL
        cache = ProviderCache(Path(tmpdir), ttl_days=0)
        cache.ttl_seconds = 0.1  # 100ms

        data = {"id": 123, "title": "Test Movie"}
        cache.set("tmdb", "movie", "123", data)

        # Should exist immediately
        assert cache.get("tmdb", "movie", "123") == data

        # Wait for expiration
        time.sleep(0.2)

        # Should be expired
        assert cache.get("tmdb", "movie", "123") is None


def test_cache_clear_expired():
    """Test clearing expired entries."""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache = ProviderCache(Path(tmpdir), ttl_days=0)
        cache.ttl_seconds = 0.1

        # Add entries
        cache.set("tmdb", "movie", "1", {"id": 1})
        cache.set("tmdb", "movie", "2", {"id": 2})

        # Wait for expiration
        time.sleep(0.2)

        # Clear expired
        count = cache.clear_expired()
        assert count == 2

        # Should be gone
        assert cache.get("tmdb", "movie", "1") is None
        assert cache.get("tmdb", "movie", "2") is None


def test_cache_clear_all():
    """Test clearing all cache entries."""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache = ProviderCache(Path(tmpdir), ttl_days=7)

        cache.set("tmdb", "movie", "1", {"id": 1})
        cache.set("tmdb", "tv", "2", {"id": 2})

        cache.clear_all()

        assert cache.get("tmdb", "movie", "1") is None
        assert cache.get("tmdb", "tv", "2") is None


def test_cache_stats():
    """Test cache statistics."""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache = ProviderCache(Path(tmpdir), ttl_days=0)
        cache.ttl_seconds = 0.1

        # Add entries
        cache.set("tmdb", "movie", "1", {"id": 1})
        cache.set("tmdb", "movie", "2", {"id": 2})

        stats = cache.get_stats()
        assert stats["total"] == 2
        assert stats["expired"] == 0

        # Wait for expiration
        time.sleep(0.2)

        stats = cache.get_stats()
        assert stats["total"] == 2
        assert stats["expired"] == 2


def test_cache_list_data():
    """Test caching list data (search results)."""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache = ProviderCache(Path(tmpdir), ttl_days=7)

        data = [{"id": 1, "title": "Movie 1"}, {"id": 2, "title": "Movie 2"}]
        cache.set("tmdb", "search_movie", "matrix", data)

        retrieved = cache.get("tmdb", "search_movie", "matrix")
        assert retrieved == data
        assert isinstance(retrieved, list)
