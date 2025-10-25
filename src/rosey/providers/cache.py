"""Disk-backed cache for provider metadata."""

import json
import sqlite3
import time
from pathlib import Path
from threading import Lock
from typing import Any


class ProviderCache:
    """SQLite-backed cache for provider metadata."""

    def __init__(self, cache_dir: Path, ttl_days: int = 30):
        """Initialize cache.

        Args:
            cache_dir: Directory for cache database
            ttl_days: Time-to-live in days for cached entries
        """
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        self.db_path = self.cache_dir / "provider_cache.db"
        self.ttl_seconds = float(ttl_days * 24 * 3600)
        self._lock = Lock()
        self._init_db()

    def _init_db(self) -> None:
        """Initialize database schema."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS cache (
                    provider TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    key TEXT NOT NULL,
                    data TEXT NOT NULL,
                    updated_at REAL NOT NULL,
                    PRIMARY KEY (provider, kind, key)
                )
                """
            )
            conn.execute("CREATE INDEX IF NOT EXISTS idx_updated_at ON cache(updated_at)")
            conn.commit()

    def get(
        self, provider: str, kind: str, key: str
    ) -> dict[str, Any] | list[dict[str, Any]] | None:
        """Get cached data if not expired.

        Args:
            provider: Provider name (e.g., 'tmdb', 'tvdb')
            kind: Data kind (e.g., 'movie', 'tv', 'search_movie')
            key: Lookup key (e.g., ID or search query)

        Returns:
            Cached data or None if not found/expired
        """
        with self._lock, sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                """
                    SELECT data, updated_at FROM cache
                    WHERE provider = ? AND kind = ? AND key = ?
                    """,
                (provider, kind, key),
            )
            row = cursor.fetchone()

            if not row:
                return None

            data_json, updated_at = row

            # Check if expired
            if time.time() - updated_at > self.ttl_seconds:
                # Delete expired entry
                conn.execute(
                    """
                        DELETE FROM cache
                        WHERE provider = ? AND kind = ? AND key = ?
                        """,
                    (provider, kind, key),
                )
                conn.commit()
                return None

            return json.loads(data_json)  # type: ignore[no-any-return]

    def set(
        self,
        provider: str,
        kind: str,
        key: str,
        data: dict[str, Any] | list[dict[str, Any]],
    ) -> None:
        """Store data in cache.

        Args:
            provider: Provider name
            kind: Data kind
            key: Lookup key
            data: Data to cache
        """
        with self._lock, sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
                    INSERT OR REPLACE INTO cache (provider, kind, key, data, updated_at)
                    VALUES (?, ?, ?, ?, ?)
                    """,
                (provider, kind, key, json.dumps(data), time.time()),
            )
            conn.commit()

    def clear_expired(self) -> int:
        """Remove expired entries from cache.

        Returns:
            Number of entries removed
        """
        with self._lock, sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                "DELETE FROM cache WHERE updated_at < ?",
                (time.time() - self.ttl_seconds,),
            )
            conn.commit()
            return cursor.rowcount

    def clear_all(self) -> None:
        """Clear all cached data."""
        with self._lock, sqlite3.connect(self.db_path) as conn:
            conn.execute("DELETE FROM cache")
            conn.commit()

    def get_stats(self) -> dict[str, int]:
        """Get cache statistics.

        Returns:
            Dictionary with total entries and expired entries
        """
        with self._lock, sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute("SELECT COUNT(*) FROM cache")
            total = cursor.fetchone()[0]

            cursor = conn.execute(
                "SELECT COUNT(*) FROM cache WHERE updated_at < ?",
                (time.time() - self.ttl_seconds,),
            )
            expired = cursor.fetchone()[0]

            return {"total": total, "expired": expired}
