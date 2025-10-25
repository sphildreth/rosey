# Rosey Test Fixtures

This directory contains recorded API responses for testing provider integrations without hitting live APIs.

## Directory Structure

```
tests/fixtures/
├── providers/
│   ├── tmdb/
│   │   ├── search_movie_the_matrix.json
│   │   ├── movie_603_en-US.json
│   │   └── ...
│   └── tvdb/
│       ├── search_series_the_office.json
│       ├── episodes_series_73244_season_2.json
│       └── ...
└── README.md (this file)
```

## Recording Strategy

### Default Behavior (CI and local development)

By default, tests use **recorded fixtures** from this directory. If a fixture is missing, the test fails with a clear error message indicating which fixture needs to be recorded.

This ensures:
- Fast, deterministic tests
- No API rate limit concerns in CI
- No API keys needed for contributors
- Reproducible test results

### Recording New Fixtures

To record new fixtures or update existing ones:

1. Set environment variable:
   ```bash
   export ROSEY_RECORD_FIXTURES=1
   ```

2. Provide API keys:
   ```bash
   export TMDB_API_KEY=your_tmdb_key
   export TVDB_API_KEY=your_tvdb_key
   ```

3. Run tests:
   ```bash
   pytest tests/integration/test_providers.py
   ```

4. Review and commit new/updated fixture files

### Fixture Format

Each fixture is a JSON file containing:

```json
{
  "request": {
    "method": "GET",
    "url": "https://api.themoviedb.org/3/search/movie",
    "params": {"query": "The Matrix", "language": "en-US"},
    "headers": {}
  },
  "response": {
    "status": 200,
    "headers": {"Content-Type": "application/json"},
    "body": {
      "results": [
        {
          "id": 603,
          "title": "The Matrix",
          "release_date": "1999-03-30",
          ...
        }
      ]
    }
  },
  "metadata": {
    "recorded_at": "2025-10-25T12:00:00Z",
    "expires_at": "2026-10-25T12:00:00Z",
    "provider": "tmdb",
    "description": "TMDB search for 'The Matrix'"
  }
}
```

### Fixture Matching

Fixtures are matched by:
1. **Provider**: tmdb or tvdb
2. **Endpoint**: e.g., `search/movie`, `movie/{id}`
3. **Parameters**: Normalized and sorted query parameters

The fixture filename is derived from these components, e.g.:
- `search_movie_the_matrix.json`
- `movie_603_en-US.json`
- `search_series_the_office.json`

### Sensitive Data Handling

**IMPORTANT**: API keys and personal tokens are never stored in fixtures. Only public response data is recorded.

Before committing fixtures:
1. Review for any sensitive information
2. Remove any user-specific data
3. Ensure only public API responses are included

## Test Guidelines

### Writing Tests with Fixtures

```python
import pytest
from rosey.providers.tmdb import TMDBProvider

@pytest.mark.asyncio
async def test_search_movie_with_fixture():
    """Test TMDB movie search using recorded fixture."""
    provider = TMDBProvider(api_key="test_key", use_fixtures=True)
    results = await provider.search_movie("The Matrix")

    assert len(results) > 0
    assert results[0]["title"] == "The Matrix"
    assert results[0]["id"] == 603

@pytest.mark.live
@pytest.mark.skipif(not os.getenv("TMDB_API_KEY"), reason="No API key provided")
async def test_search_movie_live():
    """Test TMDB movie search with live API (requires key)."""
    api_key = os.getenv("TMDB_API_KEY")
    provider = TMDBProvider(api_key=api_key, use_fixtures=False)
    results = await provider.search_movie("The Matrix")

    assert len(results) > 0
```

### Running Live Tests

Live tests are skipped by default. To run them:

```bash
# Provide API keys
export TMDB_API_KEY=your_key
export TVDB_API_KEY=your_key

# Run only live tests
pytest -m live

# Run all tests including live
pytest -m "not slow"
```

## Fixture Maintenance

### When to Update Fixtures

Update fixtures when:
- API response format changes
- New fields are added to provider responses
- Test coverage requires new scenarios
- Provider deprecates old endpoints

### Expiration Policy

Fixtures include an `expires_at` field. Expired fixtures trigger a warning but don't fail tests automatically. This allows gradual refresh of fixtures.

To find expired fixtures:
```bash
pytest tests/integration/test_providers.py --fixtures-check-expiry
```

### Pruning Unused Fixtures

To identify unused fixtures:
```bash
pytest tests/integration/test_providers.py --fixtures-check-usage
```

Remove fixtures that are no longer referenced by any test.

## CI Configuration

In CI environments, tests run with:
- `ROSEY_RECORD_FIXTURES=0` (default, use fixtures)
- No API keys provided
- `--fixtures-strict` flag to fail on missing fixtures

This ensures:
- Consistent test results
- Fast CI runs
- No API rate limit issues
- Clear indication when new fixtures are needed

## Troubleshooting

### Missing Fixture Error

```
FixtureNotFoundError: Fixture not found: tests/fixtures/providers/tmdb/search_movie_inception.json
To record this fixture, run with ROSEY_RECORD_FIXTURES=1 and provide API keys.
```

**Solution**: Record the fixture as described above.

### Expired Fixture Warning

```
FixtureExpiredWarning: Fixture expired: tests/fixtures/providers/tmdb/movie_603_en-US.json
Consider re-recording this fixture with ROSEY_RECORD_FIXTURES=1.
```

**Solution**: Re-record the fixture if provider response may have changed. If the fixture is still valid, update the `expires_at` field.

### Live Test Failures

If live tests fail but fixture tests pass, this indicates the provider API has changed. Record new fixtures and update tests accordingly.

## Best Practices

1. **Keep fixtures minimal**: Only include data needed for tests
2. **Use descriptive names**: Fixture names should clearly indicate what they contain
3. **Group by provider**: Organize fixtures by provider and endpoint
4. **Version control**: Commit fixtures to git for reproducibility
5. **Review before commit**: Ensure no sensitive data is included
6. **Update regularly**: Refresh fixtures periodically to catch API changes
7. **Document coverage**: Add comments in tests explaining which scenarios are covered

## Future Enhancements

- Automatic fixture expiration checks in CI
- Fixture validation against OpenAPI schemas
- Fixture generation from provider documentation
- Fixture comparison tool for spotting API changes
