#!/usr/bin/env python3
"""
Demo script showing TMDB ID extraction from directory paths.

This demonstrates the new feature where Rosey can extract TMDB IDs
from directory names using the [tmdbid-XXXX] pattern and use the
TMDB API to definitively identify whether content is a movie or TV show.
"""

from rosey.identifier.patterns import extract_tmdb_id_from_path

print("=" * 70)
print("TMDB ID Directory Pattern Extraction Demo")
print("=" * 70)

# Example paths
movie_path = "/movies/The Matrix (1999) [tmdbid-603]/The Matrix.mkv"
tv_path = "/tv/Breaking Bad (2008) [tmdbid-1396]/Season 01/S01E01.mkv"
no_tmdb_path = "/movies/Some Random Movie/movie.mkv"

print("\n1. Pattern Extraction Examples")
print("-" * 70)

print(f"\nMovie path: {movie_path}")
print(f"Extracted TMDB ID: {extract_tmdb_id_from_path(movie_path)}")

print(f"\nTV path: {tv_path}")
print(f"Extracted TMDB ID: {extract_tmdb_id_from_path(tv_path)}")

print(f"\nNo TMDB ID: {no_tmdb_path}")
print(f"Extracted TMDB ID: {extract_tmdb_id_from_path(no_tmdb_path)}")

print("\n2. Identification with TMDB Provider")
print("-" * 70)
print("\nTo use TMDB API for definitive identification, initialize with provider:")
print(
    """
from rosey.providers.tmdb import TMDBProvider

provider = TMDBProvider(api_key="your_api_key")
identifier = Identifier(tmdb_provider=provider)

# Movie example
result = identifier.identify("/movies/The Matrix (1999) [tmdbid-603]/movie.mkv")
print(f"Kind: {result.item.kind}")  # → "movie"
print(f"TMDB ID: {result.item.nfo.get('tmdbid')}")  # → "603"

# TV show example
result = identifier.identify("/tv/Breaking Bad (2008) [tmdbid-1396]/S01E01.mkv")
print(f"Kind: {result.item.kind}")  # → "episode"
print(f"TMDB ID: {result.item.nfo.get('tmdbid')}")  # → "1396"
"""
)

print("\n3. Benefits")
print("-" * 70)
print(
    """
✓ Definitive identification using TMDB API
✓ Works for ambiguous cases (folders without clear episode patterns)
✓ Caches API results to avoid redundant queries
✓ Falls back gracefully when provider not available
✓ Complements existing NFO-based identification
"""
)

print("=" * 70)
