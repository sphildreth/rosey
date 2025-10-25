"""Tests for provider manager."""

import json
import tempfile
from pathlib import Path
from unittest.mock import patch

import pytest

from rosey.providers.manager import ProviderManager


@pytest.fixture
def fixtures_dir():
    """Get fixtures directory."""
    return Path(__file__).parent.parent / "fixtures" / "providers"


@pytest.fixture
def provider_manager():
    """Create provider manager with temp cache."""
    with tempfile.TemporaryDirectory() as tmpdir:
        manager = ProviderManager(cache_dir=Path(tmpdir), cache_ttl_days=7, enabled=True)
        yield manager
        manager.close()


def test_manager_disabled_returns_empty(provider_manager):
    """Test that disabled manager returns empty results."""
    provider_manager.enabled = False

    results = provider_manager.search_movie("The Matrix")
    assert results == []

    result = provider_manager.get_movie_by_id("123")
    assert result is None


def test_manager_no_provider_configured(provider_manager):
    """Test manager with no provider configured."""
    # Manager is enabled but no TMDB configured
    results = provider_manager.search_movie("The Matrix")
    assert results == []


def test_manager_search_movie_with_cache(provider_manager, fixtures_dir):
    """Test movie search with caching."""
    with open(fixtures_dir / "tmdb_search_movie_matrix.json") as f:
        fixture_data = json.load(f)

    # Configure TMDB
    provider_manager.configure_tmdb("test_key", "en-US", "US")

    with patch.object(provider_manager._tmdb, "_request") as mock_request:
        mock_request.return_value = fixture_data

        # First call - should hit provider
        results1 = provider_manager.search_movie("The Matrix", 1999)
        assert len(results1) == 1
        assert results1[0]["id"] == 603
        assert mock_request.call_count == 1

        # Second call - should hit cache
        results2 = provider_manager.search_movie("The Matrix", 1999)
        assert results2 == results1
        assert mock_request.call_count == 1  # No additional call


def test_manager_search_tv_with_cache(provider_manager, fixtures_dir):
    """Test TV search with caching."""
    with open(fixtures_dir / "tmdb_search_tv_office.json") as f:
        fixture_data = json.load(f)

    provider_manager.configure_tmdb("test_key")

    with patch.object(provider_manager._tmdb, "_request") as mock_request:
        mock_request.return_value = fixture_data

        results = provider_manager.search_tv("The Office")
        assert len(results) == 1
        assert results[0]["id"] == 2316


def test_manager_get_movie_by_id(provider_manager, fixtures_dir):
    """Test getting movie by ID."""
    with open(fixtures_dir / "tmdb_movie_603.json") as f:
        fixture_data = json.load(f)

    provider_manager.configure_tmdb("test_key")

    with patch.object(provider_manager._tmdb, "_request") as mock_request:
        mock_request.return_value = fixture_data

        result = provider_manager.get_movie_by_id("603")
        assert result is not None
        assert result["id"] == 603
        assert result["title"] == "The Matrix"


def test_manager_get_tv_by_id(provider_manager, fixtures_dir):
    """Test getting TV show by ID."""
    with open(fixtures_dir / "tmdb_tv_2316.json") as f:
        fixture_data = json.load(f)

    provider_manager.configure_tmdb("test_key")

    with patch.object(provider_manager._tmdb, "_request") as mock_request:
        mock_request.return_value = fixture_data

        result = provider_manager.get_tv_by_id("2316")
        assert result is not None
        assert result["id"] == 2316
        assert result["name"] == "The Office"


def test_manager_get_episode(provider_manager, fixtures_dir):
    """Test getting episode details."""
    with open(fixtures_dir / "tmdb_episode_2316_s02e01.json") as f:
        fixture_data = json.load(f)

    provider_manager.configure_tmdb("test_key")

    with patch.object(provider_manager._tmdb, "_request") as mock_request:
        mock_request.return_value = fixture_data

        result = provider_manager.get_episode("2316", 2, 1)
        assert result is not None
        assert result["name"] == "The Dundies"


def test_manager_graceful_degradation(provider_manager):
    """Test graceful degradation on provider errors."""
    provider_manager.configure_tmdb("test_key")

    with patch.object(provider_manager._tmdb, "search_movie") as mock_search:
        # Simulate provider exception
        mock_search.side_effect = Exception("Network error")

        results = provider_manager.search_movie("Test")
        # Should return empty, not raise
        assert results == []


def test_manager_skip_cache(provider_manager, fixtures_dir):
    """Test skipping cache."""
    with open(fixtures_dir / "tmdb_search_movie_matrix.json") as f:
        fixture_data = json.load(f)

    provider_manager.configure_tmdb("test_key")

    with patch.object(provider_manager._tmdb, "_request") as mock_request:
        mock_request.return_value = fixture_data

        # First call without cache
        _ = provider_manager.search_movie("The Matrix", use_cache=False)
        assert mock_request.call_count == 1

        # Second call without cache - should call provider again
        _ = provider_manager.search_movie("The Matrix", use_cache=False)
        assert mock_request.call_count == 2


def test_manager_reconfigure_provider(provider_manager):
    """Test reconfiguring provider with different settings."""
    # First config
    provider_manager.configure_tmdb("key1", "en-US", "US")
    assert provider_manager._tmdb.api_key == "key1"
    assert provider_manager._tmdb.language == "en-US"

    # Reconfigure
    provider_manager.configure_tmdb("key2", "fr-FR", "FR")
    assert provider_manager._tmdb.api_key == "key2"
    assert provider_manager._tmdb.language == "fr-FR"


def test_manager_close(provider_manager):
    """Test closing manager closes providers."""
    provider_manager.configure_tmdb("test_key")

    with patch.object(provider_manager._tmdb, "close") as mock_close:
        provider_manager.close()
        mock_close.assert_called_once()
