"""Tests for TMDB provider using recorded fixtures."""

import json
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from rosey.providers.tmdb import TMDBProvider


@pytest.fixture
def fixtures_dir():
    """Get fixtures directory."""
    return Path(__file__).parent.parent / "fixtures" / "providers"


@pytest.fixture
def tmdb_provider():
    """Create TMDB provider with test API key."""
    return TMDBProvider(api_key="test_key", language="en-US", region="US")


def test_tmdb_search_movie_cached(tmdb_provider, fixtures_dir):
    """Test movie search with cached fixture."""
    with open(fixtures_dir / "tmdb_search_movie_matrix.json") as f:
        fixture_data = json.load(f)

    with patch.object(tmdb_provider._client, "get") as mock_get:
        mock_response = MagicMock()
        mock_response.json.return_value = fixture_data
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        results = tmdb_provider.search_movie("The Matrix", 1999)

        assert len(results) == 1
        assert results[0]["id"] == 603
        assert results[0]["title"] == "The Matrix"
        assert results[0]["release_date"] == "1999-03-30"


def test_tmdb_search_tv_cached(tmdb_provider, fixtures_dir):
    """Test TV search with cached fixture."""
    with open(fixtures_dir / "tmdb_search_tv_office.json") as f:
        fixture_data = json.load(f)

    with patch.object(tmdb_provider._client, "get") as mock_get:
        mock_response = MagicMock()
        mock_response.json.return_value = fixture_data
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        results = tmdb_provider.search_tv("The Office")

        assert len(results) == 1
        assert results[0]["id"] == 2316
        assert results[0]["name"] == "The Office"


def test_tmdb_get_movie_by_id_cached(tmdb_provider, fixtures_dir):
    """Test get movie by ID with cached fixture."""
    with open(fixtures_dir / "tmdb_movie_603.json") as f:
        fixture_data = json.load(f)

    with patch.object(tmdb_provider._client, "get") as mock_get:
        mock_response = MagicMock()
        mock_response.json.return_value = fixture_data
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        result = tmdb_provider.get_movie_by_id("603")

        assert result is not None
        assert result["id"] == 603
        assert result["title"] == "The Matrix"
        assert result["imdb_id"] == "tt0133093"


def test_tmdb_get_tv_by_id_cached(tmdb_provider, fixtures_dir):
    """Test get TV show by ID with cached fixture."""
    with open(fixtures_dir / "tmdb_tv_2316.json") as f:
        fixture_data = json.load(f)

    with patch.object(tmdb_provider._client, "get") as mock_get:
        mock_response = MagicMock()
        mock_response.json.return_value = fixture_data
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        result = tmdb_provider.get_tv_by_id("2316")

        assert result is not None
        assert result["id"] == 2316
        assert result["name"] == "The Office"
        assert result["number_of_seasons"] == 9


def test_tmdb_get_episode_cached(tmdb_provider, fixtures_dir):
    """Test get episode with cached fixture."""
    with open(fixtures_dir / "tmdb_episode_2316_s02e01.json") as f:
        fixture_data = json.load(f)

    with patch.object(tmdb_provider._client, "get") as mock_get:
        mock_response = MagicMock()
        mock_response.json.return_value = fixture_data
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        result = tmdb_provider.get_episode("2316", 2, 1)

        assert result is not None
        assert result["name"] == "The Dundies"
        assert result["season_number"] == 2
        assert result["episode_number"] == 1


def test_tmdb_http_error_graceful():
    """Test graceful handling of HTTP errors."""
    provider = TMDBProvider(api_key="test_key")

    with patch.object(provider._client, "get") as mock_get:
        from httpx import HTTPStatusError, Request, Response

        # Simulate 404 error
        mock_request = Request("GET", "http://test")
        mock_response = MagicMock(spec=Response)
        mock_response.status_code = 404
        mock_response.request = mock_request
        mock_response.raise_for_status.side_effect = HTTPStatusError(
            "Not found", request=mock_request, response=mock_response
        )
        mock_get.return_value = mock_response

        result = provider.search_movie("NonexistentMovie")

        assert result == []


def test_tmdb_rate_limiting():
    """Test rate limiting doesn't crash."""
    provider = TMDBProvider(api_key="test_key")

    # Make multiple requests quickly
    with patch.object(provider._client, "get") as mock_get:
        mock_response = MagicMock()
        mock_response.json.return_value = {"results": []}
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        # Should not crash
        for _ in range(10):
            provider.search_movie("Test")

        # At least some calls should have been made
        assert mock_get.call_count > 0


def test_tmdb_empty_results(tmdb_provider):
    """Test handling of empty search results."""
    with patch.object(tmdb_provider._client, "get") as mock_get:
        mock_response = MagicMock()
        mock_response.json.return_value = {"results": [], "total_results": 0}
        mock_response.raise_for_status = MagicMock()
        mock_get.return_value = mock_response

        results = tmdb_provider.search_movie("XYZ123NonExistent")

        assert results == []


@pytest.mark.skip(reason="Requires valid TMDB_API_KEY env var and --run-live flag")
def test_tmdb_live_search():
    """Live test with real TMDB API (requires valid key and --run-live flag)."""
    import os

    api_key = os.environ.get("TMDB_API_KEY")
    if not api_key:
        pytest.skip("TMDB_API_KEY not set")

    assert api_key is not None  # Type narrowing for mypy
    provider = TMDBProvider(api_key=api_key)

    results = provider.search_movie("The Matrix", 1999)

    assert len(results) > 0
    # Should find The Matrix
    assert any("Matrix" in r.get("title", "") for r in results)

    provider.close()
