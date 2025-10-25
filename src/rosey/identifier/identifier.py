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
                file_path, filename, folder_name, parent_folder, episode_info, date_info, nfo_data, reasons
            )
        elif nfo_data and (nfo_data.tmdb_id or nfo_data.imdb_id) and not nfo_data.season:
            # NFO with TMDB/IMDB ID but no season = movie
            item = self._identify_movie(file_path, filename, nfo_data, reasons)
        else:
            # Try to identify as movie (default for video files without episode markers)
            year = extract_year(filename) or extract_year(folder_name)
            part = extract_part(filename)

            if year or part or (nfo_data and nfo_data.year):
                # Has year or part -> likely movie
                item = self._identify_movie(file_path, filename, nfo_data, reasons)
            elif nfo_data and nfo_data.title:
                # Has NFO with title -> assume movie
                item = self._identify_movie(file_path, filename, nfo_data, reasons)
            else:
                # Default to movie for unidentified video files
                item = self._identify_movie(file_path, filename, nfo_data, reasons)
                reasons.append("No clear pattern - defaulting to movie")

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
            # Try to extract from parent folders
            title = clean_title(parent_folder or folder_name)
            reasons.append("Show title from folder structure")

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

        return MediaItem(
            kind="episode",
            source_path=file_path,
            title=title,
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
            title = clean_title(filename)
            year = extract_year(filename)
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
