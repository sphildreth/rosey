"""Tests for TVDB provider using mocked responses."""

from unittest.mock import MagicMock, patch

import pytest

from rosey.providers.tvdb import TVDBProvider


@pytest.fixture
def tvdb_provider():
    """Create TVDB provider with test API key."""
    return TVDBProvider(api_key="test_key", language="eng")


def test_tvdb_init(tvdb_provider):
    """Test TVDB provider initialization."""
    assert tvdb_provider.api_key == "test_key"
    assert tvdb_provider.language == "eng"
    assert tvdb_provider.timeout == 10.0
    assert tvdb_provider._token is None
    assert tvdb_provider._token_expires == 0


def test_tvdb_search_movie(tvdb_provider):
    """Test movie search."""
    mock_data = {
        "data": [
            {
                "id": "12345",
                "name": "Test Movie",
                "first_air_time": "2020-01-01",
                "objectID": "movie-12345",
            }
        ]
    }

    with patch.object(tvdb_provider, "_request") as mock_request:
        mock_request.return_value = mock_data

        results = tvdb_provider.search_movie("Test Movie", 2020)

        mock_request.assert_called_once_with(
            "/search", {"query": "Test Movie", "type": "movie", "year": "2020"}
        )
        assert len(results) == 1
        assert results[0]["id"] == "12345"
        assert results[0]["name"] == "Test Movie"


def test_tvdb_search_tv(tvdb_provider):
    """Test TV show search."""
    mock_data = {
        "data": [
            {
                "id": "67890",
                "name": "Test Show",
                "first_air_time": "2019-01-01",
                "objectID": "series-67890",
            }
        ]
    }

    with patch.object(tvdb_provider, "_request") as mock_request:
        mock_request.return_value = mock_data

        results = tvdb_provider.search_tv("Test Show")

        mock_request.assert_called_once_with("/search", {"query": "Test Show", "type": "series"})
        assert len(results) == 1
        assert results[0]["id"] == "67890"
        assert results[0]["name"] == "Test Show"


def test_tvdb_get_movie_by_id(tvdb_provider):
    """Test get movie by ID."""
    mock_data = {
        "data": {
            "id": "12345",
            "name": "Test Movie",
            "overview": "A test movie",
            "first_air_time": "2020-01-01",
        }
    }

    with patch.object(tvdb_provider, "_request") as mock_request:
        mock_request.return_value = mock_data

        result = tvdb_provider.get_movie_by_id("12345")

        mock_request.assert_called_once_with("/movies/12345/extended")
        assert result is not None
        assert result["id"] == "12345"
        assert result["name"] == "Test Movie"


def test_tvdb_get_tv_by_id(tvdb_provider):
    """Test get TV show by ID."""
    mock_data = {
        "data": {
            "id": "67890",
            "name": "Test Show",
            "overview": "A test show",
            "first_air_time": "2019-01-01",
        }
    }

    with patch.object(tvdb_provider, "_request") as mock_request:
        mock_request.return_value = mock_data

        result = tvdb_provider.get_tv_by_id("67890")

        mock_request.assert_called_once_with("/series/67890/extended")
        assert result is not None
        assert result["id"] == "67890"
        assert result["name"] == "Test Show"


def test_tvdb_get_episode(tvdb_provider):
    """Test get episode details."""
    mock_data = {
        "data": {
            "episodes": [
                {
                    "id": "111",
                    "name": "Pilot",
                    "seasonNumber": 1,
                    "number": 1,
                    "overview": "First episode",
                },
                {
                    "id": "222",
                    "name": "Episode 2",
                    "seasonNumber": 1,
                    "number": 2,
                    "overview": "Second episode",
                },
            ]
        }
    }

    with patch.object(tvdb_provider, "_request") as mock_request:
        mock_request.return_value = mock_data

        result = tvdb_provider.get_episode("67890", 1, 2)

        mock_request.assert_called_once_with("/series/67890/episodes/default")
        assert result is not None
        assert result["id"] == "222"
        assert result["name"] == "Episode 2"
        assert result["seasonNumber"] == 1
        assert result["number"] == 2


def test_tvdb_get_episode_not_found(tvdb_provider):
    """Test get episode when not found."""
    mock_data = {
        "data": {"episodes": [{"id": "111", "name": "Pilot", "seasonNumber": 1, "number": 1}]}
    }

    with patch.object(tvdb_provider, "_request") as mock_request:
        mock_request.return_value = mock_data

        result = tvdb_provider.get_episode("67890", 1, 2)

        assert result is None


def test_tvdb_request_failure(tvdb_provider):
    """Test request failure handling."""
    with patch.object(tvdb_provider, "_request") as mock_request:
        mock_request.return_value = None

        results = tvdb_provider.search_movie("Test")
        assert results == []

        result = tvdb_provider.get_movie_by_id("123")
        assert result is None


def test_tvdb_empty_results(tvdb_provider):
    """Test handling of empty results."""
    mock_data = {"data": []}

    with patch.object(tvdb_provider, "_request") as mock_request:
        mock_request.return_value = mock_data

        results = tvdb_provider.search_tv("Nonexistent")
        assert results == []


def test_tvdb_close(tvdb_provider):
    """Test closing the provider."""
    with patch.object(tvdb_provider._client, "close") as mock_close:
        tvdb_provider.close()
        mock_close.assert_called_once()


def test_tvdb_ensure_token_success(tvdb_provider):
    """Test successful token acquisition."""
    mock_response = {"data": {"token": "test_token"}}

    with patch.object(tvdb_provider._client, "post") as mock_post:
        mock_resp = MagicMock()
        mock_resp.json.return_value = mock_response
        mock_resp.raise_for_status = MagicMock()
        mock_post.return_value = mock_resp

        success = tvdb_provider._ensure_token()

        assert success is True
        assert tvdb_provider._token == "test_token"
        assert tvdb_provider._token_expires > 0


def test_tvdb_ensure_token_failure(tvdb_provider):
    """Test token acquisition failure."""
    from httpx import RequestError

    with patch.object(tvdb_provider._client, "post") as mock_post:
        mock_post.side_effect = RequestError("Network error")

        success = tvdb_provider._ensure_token()

        assert success is False
        assert tvdb_provider._token is None


def test_tvdb_request_with_token(tvdb_provider):
    """Test request with valid token."""
    tvdb_provider._token = "test_token"
    tvdb_provider._token_expires = 9999999999  # Far future

    mock_data = {"data": {"test": "data"}}

    with patch.object(tvdb_provider._client, "get") as mock_get:
        mock_resp = MagicMock()
        mock_resp.json.return_value = mock_data
        mock_resp.raise_for_status = MagicMock()
        mock_get.return_value = mock_resp

        result = tvdb_provider._request("/test")

        assert result == mock_data
        mock_get.assert_called_once_with(
            "https://api4.thetvdb.com/v4/test",
            headers={"Authorization": "Bearer test_token"},
            params=None,
        )


def test_tvdb_request_token_refresh(tvdb_provider):
    """Test token refresh when expired."""
    tvdb_provider._token = "old_token"
    tvdb_provider._token_expires = 0  # Expired

    # Mock login response
    login_response = {"data": {"token": "new_token"}}

    # Mock data response
    data_response = {"data": {"test": "data"}}

    with patch.object(tvdb_provider._client, "post") as mock_post, patch.object(
        tvdb_provider._client, "get"
    ) as mock_get:
        # Login mock
        login_resp = MagicMock()
        login_resp.json.return_value = login_response
        login_resp.raise_for_status = MagicMock()
        mock_post.return_value = login_resp

        # Data mock
        data_resp = MagicMock()
        data_resp.json.return_value = data_response
        data_resp.raise_for_status = MagicMock()
        mock_get.return_value = data_resp

        result = tvdb_provider._request("/test")

        assert result == data_response
        assert tvdb_provider._token == "new_token"
        mock_post.assert_called_once()
        mock_get.assert_called_once()


def test_tvdb_request_no_token(tvdb_provider):
    """Test request when token acquisition fails."""
    with patch.object(tvdb_provider, "_ensure_token") as mock_ensure:
        mock_ensure.return_value = False

        result = tvdb_provider._request("/test")

        assert result is None
