"""Media identification from filenames and NFO files."""

import logging
from pathlib import Path
from typing import TYPE_CHECKING

from rosey.identifier.nfo import NFOData, find_nfo_for_file, parse_nfo
from rosey.identifier.patterns import (
    clean_title,
    extract_date,
    extract_episode_info,
    extract_part,
    extract_season_from_folder,
    extract_title_before_episode,
    extract_year,
)
from rosey.models import IdentificationResult, MediaItem

if TYPE_CHECKING:
    from rosey.identifier.patterns import DateMatch, EpisodeMatch

logger = logging.getLogger(__name__)


class Identifier:
    """Identifies media files from filesystem information."""

    def __init__(self, prefer_nfo: bool = True):
        """
        Initialize identifier.

        Args:
            prefer_nfo: Whether to prefer NFO data over filename parsing
        """
        self.prefer_nfo = prefer_nfo

    def identify(self, file_path: str) -> IdentificationResult:
        """
        Identify a media file.

        Args:
            file_path: Path to media file

        Returns:
            IdentificationResult with identified media info
        """
        path = Path(file_path)
        reasons = []
        errors = []

        # Try to find and parse NFO
        nfo_data = None
        nfo_path = find_nfo_for_file(file_path)
        if nfo_path:
            nfo_data = parse_nfo(nfo_path)
            if nfo_data:
                reasons.append(f"Found NFO file: {Path(nfo_path).name}")
            else:
                errors.append(f"Failed to parse NFO: {Path(nfo_path).name}")

        # Parse filename and folder structure
        filename = path.stem  # Without extension
        folder_name = path.parent.name
        parent_folder = path.parent.parent.name

        # Try episode identification first
        episode_info = extract_episode_info(filename)
        if not episode_info:
            # Check folder names for season info
            season = extract_season_from_folder(folder_name) or extract_season_from_folder(
                parent_folder
            )
            if season:
                episode_info = extract_episode_info(folder_name)

        # Check for date-based episode
        date_info = extract_date(filename)

        # Determine media type and build MediaItem
        if episode_info or date_info or (nfo_data and nfo_data.season is not None):
            # This is a TV episode
            item = self._identify_episode(
                file_path,
                filename,
                folder_name,
                parent_folder,
                episode_info,
                date_info,
                nfo_data,
                reasons,
            )
        elif nfo_data and (nfo_data.tmdb_id or nfo_data.imdb_id) and not nfo_data.season:
            # NFO with TMDB/IMDB ID but no season = movie
            item = self._identify_movie(file_path, filename, nfo_data, reasons)
        else:
            # Check if filename suggests it's a movie
            year = extract_year(filename) or extract_year(folder_name)
            part = extract_part(filename)

            # Check for movie-like keywords in filename
            movie_keywords = ["episode", "invalid date"]
            filename_lower = filename.lower()
            has_movie_keyword = any(keyword in filename_lower for keyword in movie_keywords)

            if year or part or (nfo_data and nfo_data.year):
                # Has year or part -> likely movie
                item = self._identify_movie(file_path, filename, nfo_data, reasons)
            elif nfo_data and nfo_data.title:
                # Has NFO with title -> assume movie
                item = self._identify_movie(file_path, filename, nfo_data, reasons)
            elif has_movie_keyword:
                # Has movie keyword -> assume movie
                item = self._identify_movie(file_path, filename, nfo_data, reasons)
                reasons.append("Movie keyword detected")
            else:
                # Default to movie for unidentified video files without episode markers
                item = self._identify_movie(file_path, filename, nfo_data, reasons)
                reasons.append("No episode pattern - defaulting to movie")

        return IdentificationResult(item=item, reasons=reasons, errors=errors)

    def _identify_episode(
        self,
        file_path: str,
        filename: str,
        folder_name: str,
        parent_folder: str,
        episode_info: "EpisodeMatch | None",
        date_info: "DateMatch | None",
        nfo_data: NFOData | None,
        reasons: list[str],
    ) -> MediaItem:
        """Identify a TV episode."""
        # Determine show title
        title = None
        if nfo_data and nfo_data.title:
            title = nfo_data.title
            reasons.append("Show title from NFO")
        else:
            # Prefer a sensible folder-derived title; avoid generic roots like 'source'/'tv'
            generic_dirs = {
                "source",
                "sources",
                "tv",
                "movies",
                "movie",
                "video",
                "videos",
                "media",
                "downloads",
                "download",
                "incoming",
                "complete",
            }

            parent_clean = clean_title(parent_folder)
            folder_clean = clean_title(folder_name)
            # For filename, only use part before episode marker to avoid episode titles
            file_title_part = extract_title_before_episode(filename)
            file_clean = clean_title(file_title_part)

            is_season_dir = extract_season_from_folder(folder_name) is not None

            # Heuristics:
            # - If folder is a season directory, use the parent folder (show folder) when it's not generic
            # - Else prefer folder name if not generic
            # - Else use parent if not generic
            # - Else fall back to filename
            if is_season_dir and parent_clean and parent_folder.lower() not in generic_dirs:
                title = parent_clean
                reasons.append("Show title from folder structure")
            elif folder_clean and folder_name and folder_name.lower() not in generic_dirs:
                title = folder_clean
                reasons.append("Show title from folder structure")
            elif parent_clean and parent_folder and parent_folder.lower() not in generic_dirs:
                title = parent_clean
                reasons.append("Show title from folder structure")
            else:
                title = file_clean
                reasons.append("Show title from filename")

        # Determine season and episodes
        season = None
        episodes = None

        if episode_info:
            season = episode_info.season
            episodes = episode_info.episodes
            reasons.append(f"Parsed episode: S{season:02d}E{episodes[0]:02d}")
        elif nfo_data and nfo_data.season is not None:
            season = nfo_data.season
            if nfo_data.episode is not None:
                episodes = [nfo_data.episode]
            reasons.append("Season/episode from NFO")
        else:
            # Check folder for season
            season = extract_season_from_folder(folder_name)
            if season:
                reasons.append(f"Season {season} from folder")

        # Check for part number (multipart episode)
        part = extract_part(filename)
        if part:
            reasons.append(f"Multipart episode: Part {part}")

        # Infer show year from folder names (show folder preferred), or NFO
        year = None
        if nfo_data and nfo_data.year:
            year = nfo_data.year
        else:
            # Prefer a year embedded in the show folder (parent), else season folder
            year = extract_year(parent_folder) or extract_year(folder_name)
            if year:
                reasons.append(f"Year {year} parsed from folder")

        # Build NFO dict
        nfo_dict: dict[str, str | None] = {}
        if nfo_data:
            if nfo_data.imdb_id:
                nfo_dict["imdbid"] = nfo_data.imdb_id
                reasons.append("IMDB ID from NFO")
            if nfo_data.tmdb_id:
                nfo_dict["tmdbid"] = nfo_data.tmdb_id
                reasons.append("TMDB ID from NFO")
            if nfo_data.tvdb_id:
                nfo_dict["tvdbid"] = nfo_data.tvdb_id
                reasons.append("TVDB ID from NFO")
            if nfo_data.episode_title:
                nfo_dict["episode_title"] = nfo_data.episode_title

        # If title was derived from filename and includes a trailing parenthetical (often episode title),
        # strip it to keep just the show name. Apply conservatively when we have episode_info/date.
        # However, preserve important parentheticals like country/language markers and alternate titles
        # For now, we'll preserve all parentheticals as distinguishing information is valuable
        # if (episode_info or date_info) and title and "(" in title:
        #     # Check if the parenthetical looks like an episode title (long) vs metadata (short)
        #     paren_match = re.search(r"\(([^)]+)\)$", title)
        #     if paren_match:
        #         paren_content = paren_match.group(1)
        #         # Only remove if it's clearly an episode title
        #         if len(paren_content) > 10 or paren_content.count(' ') > 2:
        #             title = re.sub(r"\s*\([^)]*\)\s*$", "", title)

        return MediaItem(
            kind="episode",
            source_path=file_path,
            title=title,
            year=year,
            season=season,
            episodes=episodes,
            part=part,
            date=date_info.date if date_info else None,
            nfo=nfo_dict,
        )

    def _identify_movie(
        self,
        file_path: str,
        filename: str,
        nfo_data: NFOData | None,
        reasons: list[str],
    ) -> MediaItem:
        """Identify a movie."""
        # Determine title
        title = None
        year = None

        if nfo_data and nfo_data.title:
            title = nfo_data.title
            year = nfo_data.year
            reasons.append("Movie title and year from NFO")
        else:
            # Extract year first, then clean title
            year = extract_year(filename)
            title = clean_title(filename, extracted_year=year)
            reasons.append("Movie title from filename")
            if year:
                reasons.append(f"Year {year} parsed from filename")

        # Check for part number (multipart movie)
        part = extract_part(filename)
        if part:
            reasons.append(f"Multipart movie: Part {part}")

        # Build NFO dict
        nfo_dict: dict[str, str | None] = {}
        if nfo_data:
            if nfo_data.imdb_id:
                nfo_dict["imdbid"] = nfo_data.imdb_id
                reasons.append("IMDB ID from NFO")
            if nfo_data.tmdb_id:
                nfo_dict["tmdbid"] = nfo_data.tmdb_id
                reasons.append("TMDB ID from NFO")

        return MediaItem(
            kind="movie",
            source_path=file_path,
            title=title,
            year=year,
            part=part,
            nfo=nfo_dict,
        )


def identify_file(file_path: str, prefer_nfo: bool = True) -> IdentificationResult:
    """
    Convenience function to identify a single file.

    Args:
        file_path: Path to media file
        prefer_nfo: Whether to prefer NFO data over filename parsing

    Returns:
        IdentificationResult
    """
    identifier = Identifier(prefer_nfo=prefer_nfo)
    return identifier.identify(file_path)
