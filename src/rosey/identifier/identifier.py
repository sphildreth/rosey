"""Media identification from filenames and NFO files."""

import json
import logging
import subprocess
from pathlib import Path
from typing import TYPE_CHECKING

from rosey.config import RoseyConfig, load_config
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

    def __init__(
        self,
        prefer_nfo: bool = True,
        config: RoseyConfig | None = None,
        skip_duration: bool = False,
    ):
        """
        Initialize identifier.

        Args:
            prefer_nfo: Whether to prefer NFO data over filename parsing
            config: Optional config to use instead of loading from disk
            skip_duration: If True, skip expensive duration checks (faster scanning)
        """
        self.prefer_nfo = prefer_nfo
        self.config = config if config is not None else load_config()
        self.skip_duration = skip_duration

        # Performance caches
        self._duration_cache: dict[str, float | None] = {}
        self._show_folder_cache: dict[str, bool] = {}
        self._only_media_file_cache: dict[str, bool] = {}

    def _discover_companion_files(self, file_path: str) -> list[str]:
        """
        Discover companion files for a media file.

        For movies, looks for:
        - Subtitle files (.srt, .ass, .vtt) in same directory or "Subs" subdirectory
        - Image files (.jpg, .png, .jpeg) in same directory

        Args:
            file_path: Path to the media file

        Returns:
            List of companion file paths
        """
        path = Path(file_path)
        parent_dir = path.parent
        companions: list[str] = []

        if not parent_dir.exists():
            return companions

        # Extensions to look for
        subtitle_exts = {".srt", ".ass", ".vtt"}
        image_exts = {".jpg", ".png", ".jpeg"}

        try:
            # Look in parent directory
            for item in parent_dir.iterdir():
                if not item.is_file():
                    continue

                ext = item.suffix.lower()
                if ext in subtitle_exts or ext in image_exts:
                    companions.append(str(item))

            # Look in "Subs" subdirectory
            subs_dir = parent_dir / "Subs"
            if subs_dir.exists() and subs_dir.is_dir():
                for item in subs_dir.iterdir():
                    if item.is_file() and item.suffix.lower() in subtitle_exts:
                        companions.append(str(item))

        except (OSError, PermissionError):
            # If we can't read the directory, just return empty list
            pass

        return companions

    def _should_check_duration(self) -> bool:
        """Check if duration validation is enabled."""
        if self.skip_duration:
            return False
        return self.config.identification.minimum_movie_duration_minutes > 0

    def _should_check_directory_constraints(self) -> bool:
        """Check if directory constraints are enabled."""
        return self.config.identification.movies_always_in_own_directory

    def _get_video_duration_minutes(self, file_path: str) -> float | None:
        """
        Get video duration in minutes using ffprobe.

        Args:
            file_path: Path to video file

        Returns:
            Duration in minutes, or None if unable to determine
        """
        # Check cache first
        if file_path in self._duration_cache:
            return self._duration_cache[file_path]

        try:
            # Use ffprobe to get duration
            result = subprocess.run(
                [
                    "ffprobe",
                    "-v",
                    "quiet",
                    "-print_format",
                    "json",
                    "-show_format",
                    file_path,
                ],
                capture_output=True,
                text=True,
                timeout=10,
            )

            if result.returncode == 0:
                data = json.loads(result.stdout)
                duration_str = data.get("format", {}).get("duration")
                if duration_str:
                    duration_seconds = float(duration_str)
                    duration_minutes = duration_seconds / 60.0
                    self._duration_cache[file_path] = duration_minutes
                    return duration_minutes

        except (
            subprocess.TimeoutExpired,
            subprocess.SubprocessError,
            FileNotFoundError,
            json.JSONDecodeError,
            ValueError,
        ) as e:
            logger.debug(f"Failed to get duration for {file_path}: {e}")

        # Cache None for failed lookups too
        self._duration_cache[file_path] = None
        return None

    def _is_in_show_folder(self, file_path: str) -> bool:
        """
        Check if a file is located in a known TV show folder.

        A show folder is identified by:
        - Containing season subdirectories (Season X, SXX)
        - Containing multiple episode files
        - Having a parent that's a season directory
        """
        path = Path(file_path)
        directory = path.parent
        dir_path = str(directory)

        # Check cache first
        if dir_path in self._show_folder_cache:
            return self._show_folder_cache[dir_path]

        result = False

        # Check if parent directory is a season folder (quick check, no filesystem I/O)
        if extract_season_from_folder(directory.name):
            result = True
            self._show_folder_cache[dir_path] = result
            return result

        # Batch check directory contents once
        media_extensions = {".mkv", ".mp4", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v"}
        media_file_count = 0
        has_season_subdir = False

        try:
            for item in directory.iterdir():
                # Check for season subdirectories
                if item.is_dir() and extract_season_from_folder(item.name):
                    has_season_subdir = True
                    break  # Found season folder, can stop

                # Count media files (limit to 2 for efficiency)
                if item.is_file() and item.suffix.lower() in media_extensions:
                    media_file_count += 1
                    if media_file_count >= 2:
                        # Multiple media files suggest TV show
                        result = True
                        break  # Found enough evidence, can stop

            # If we found season subdirectory, it's a show folder
            if has_season_subdir:
                result = True

        except (OSError, PermissionError):
            pass

        self._show_folder_cache[dir_path] = result
        return result

    def _is_only_media_file_in_directory(self, file_path: str) -> bool:
        """
        Check if the given file is the only media file in its directory.

        Returns True if this is the only media file in the directory.
        Child directories are allowed, but no other media files in the same directory.
        """
        path = Path(file_path)
        directory = path.parent
        dir_path = str(directory)

        # Check cache first
        if dir_path in self._only_media_file_cache:
            return self._only_media_file_cache[dir_path]

        media_extensions = {".mkv", ".mp4", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v"}

        try:
            for item in directory.iterdir():
                if item.is_file() and item != path and item.suffix.lower() in media_extensions:
                    self._only_media_file_cache[dir_path] = False
                    return False  # Found another media file
        except (OSError, PermissionError):
            self._only_media_file_cache[dir_path] = True
            return True  # If we can't read the directory, assume it's okay

        self._only_media_file_cache[dir_path] = True
        return True

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
                # Try filename again with known season (for dash patterns)
                episode_info = extract_episode_info(filename, known_season=season)
                if not episode_info:
                    # Try folder name as fallback
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
            if self.config.identification.movies_always_in_own_directory:
                if self._is_in_show_folder(file_path):
                    reasons.append(
                        "File in show folder - cannot be movie when movies_always_in_own_directory is enabled"
                    )
                    item = MediaItem(
                        kind="unknown",
                        source_path=file_path,
                        title=clean_title(filename),
                        nfo={},
                    )
                elif not self._is_only_media_file_in_directory(file_path):
                    reasons.append(
                        "Directory contains multiple media files - cannot be movie when movies_always_in_own_directory is enabled"
                    )
                    item = MediaItem(
                        kind="unknown",
                        source_path=file_path,
                        title=clean_title(filename),
                        nfo={},
                    )
                else:
                    item = self._identify_movie(file_path, filename, nfo_data, reasons)
                    reasons.append("NFO with TMDB/IMDB ID and directory constraints satisfied")
            else:
                item = self._identify_movie(file_path, filename, nfo_data, reasons)
        else:
            # Check if filename suggests it's a movie
            year = extract_year(filename) or extract_year(folder_name)
            part = extract_part(filename)

            # Check for movie-like keywords in filename
            movie_keywords = ["episode", "invalid date"]
            filename_lower = filename.lower()
            has_movie_keyword = any(keyword in filename_lower for keyword in movie_keywords)

            # Determine if we need expensive checks
            needs_duration_check = self._should_check_duration()
            needs_directory_check = self._should_check_directory_constraints()

            # Get duration only if needed
            duration_minutes = None
            if needs_duration_check:
                duration_minutes = self._get_video_duration_minutes(file_path)
                min_duration = self.config.identification.minimum_movie_duration_minutes

            if year or part or (nfo_data and nfo_data.year):
                # Has year or part -> likely movie, but check constraints if enabled
                if needs_directory_check:
                    if self._is_in_show_folder(file_path):
                        reasons.append(
                            "File in show folder - cannot be movie when movies_always_in_own_directory is enabled"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                    elif not self._is_only_media_file_in_directory(file_path):
                        reasons.append(
                            "Directory contains multiple media files - cannot be movie when movies_always_in_own_directory is enabled"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                    elif duration_minutes is not None and duration_minutes < min_duration:
                        reasons.append(
                            f"Duration {duration_minutes:.1f}min < {min_duration}min - not a movie"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                        reasons.append("Short duration - classified as unknown")
                    else:
                        item = self._identify_movie(file_path, filename, nfo_data, reasons)
                        if duration_minutes is not None:
                            reasons.append(f"Duration {duration_minutes:.1f}min meets minimum")
                        reasons.append("Directory constraints and duration satisfied")
                elif (
                    needs_duration_check
                    and duration_minutes is not None
                    and duration_minutes < min_duration
                ):
                    reasons.append(
                        f"Duration {duration_minutes:.1f}min < {min_duration}min - not a movie"
                    )
                    item = MediaItem(
                        kind="unknown",
                        source_path=file_path,
                        title=clean_title(filename),
                        nfo={},
                    )
                    reasons.append("Short duration - classified as unknown")
                else:
                    item = self._identify_movie(file_path, filename, nfo_data, reasons)
                    if duration_minutes is not None:
                        reasons.append(f"Duration {duration_minutes:.1f}min meets minimum")
            elif nfo_data and nfo_data.title:
                # Has NFO with title -> assume movie, but check constraints if enabled
                if needs_directory_check:
                    if self._is_in_show_folder(file_path):
                        reasons.append(
                            "File in show folder - cannot be movie when movies_always_in_own_directory is enabled"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                    elif not self._is_only_media_file_in_directory(file_path):
                        reasons.append(
                            "Directory contains multiple media files - cannot be movie when movies_always_in_own_directory is enabled"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                    elif duration_minutes is not None and duration_minutes < min_duration:
                        reasons.append(
                            f"Duration {duration_minutes:.1f}min < {min_duration}min - not a movie"
                        )
                        item = self._identify_movie(file_path, filename, nfo_data, reasons)
                        reasons.append("Short duration despite NFO title")
                    else:
                        item = self._identify_movie(file_path, filename, nfo_data, reasons)
                        if duration_minutes is not None:
                            reasons.append(f"Duration {duration_minutes:.1f}min meets minimum")
                        reasons.append("Directory constraints and duration satisfied")
                elif (
                    needs_duration_check
                    and duration_minutes is not None
                    and duration_minutes < min_duration
                ):
                    reasons.append(
                        f"Duration {duration_minutes:.1f}min < {min_duration}min - not a movie"
                    )
                    item = self._identify_movie(file_path, filename, nfo_data, reasons)
                    reasons.append("Short duration despite NFO title")
                else:
                    item = self._identify_movie(file_path, filename, nfo_data, reasons)
                    if duration_minutes is not None:
                        reasons.append(f"Duration {duration_minutes:.1f}min meets minimum")
            elif has_movie_keyword:
                # Has movie keyword -> assume movie
                if needs_directory_check:
                    if self._is_in_show_folder(file_path):
                        reasons.append(
                            "File in show folder - cannot be movie when movies_always_in_own_directory is enabled"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                    elif not self._is_only_media_file_in_directory(file_path):
                        reasons.append(
                            "Directory contains multiple media files - cannot be movie when movies_always_in_own_directory is enabled"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                    else:
                        item = self._identify_movie(file_path, filename, nfo_data, reasons)
                        reasons.append("Movie keyword detected and directory constraints satisfied")
                else:
                    item = self._identify_movie(file_path, filename, nfo_data, reasons)
                    reasons.append("Movie keyword detected")
            else:
                # Default to movie for unidentified video files without episode markers
                # But check constraints if enabled
                if needs_directory_check:
                    if self._is_in_show_folder(file_path):
                        reasons.append(
                            "File in show folder - cannot be movie when movies_always_in_own_directory is enabled"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                    elif not self._is_only_media_file_in_directory(file_path):
                        reasons.append(
                            "Directory contains multiple media files - cannot be movie when movies_always_in_own_directory is enabled"
                        )
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                    elif duration_minutes is not None and duration_minutes < min_duration:
                        # Too short to be a movie, might be a clip/trailer/sample
                        reasons.append(
                            f"Duration {duration_minutes:.1f}min < {min_duration}min - not a movie"
                        )
                        # Classify as unknown instead of movie
                        item = MediaItem(
                            kind="unknown",
                            source_path=file_path,
                            title=clean_title(filename),
                            nfo={},
                        )
                        reasons.append("Short duration - classified as unknown")
                    else:
                        item = self._identify_movie(file_path, filename, nfo_data, reasons)
                        if duration_minutes is not None:
                            reasons.append(f"Duration {duration_minutes:.1f}min meets minimum")
                        reasons.append(
                            "No episode pattern - defaulting to movie and directory constraints satisfied"
                        )
                elif (
                    needs_duration_check
                    and duration_minutes is not None
                    and duration_minutes < min_duration
                ):
                    # Too short to be a movie, might be a clip/trailer/sample
                    reasons.append(
                        f"Duration {duration_minutes:.1f}min < {min_duration}min - not a movie"
                    )
                    # Classify as unknown instead of movie
                    item = MediaItem(
                        kind="unknown",
                        source_path=file_path,
                        title=clean_title(filename),
                        nfo={},
                    )
                    reasons.append("Short duration - classified as unknown")
                else:
                    item = self._identify_movie(file_path, filename, nfo_data, reasons)
                    if duration_minutes is not None:
                        reasons.append(f"Duration {duration_minutes:.1f}min meets minimum")
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
        episode_title_from_filename = None

        if episode_info:
            season = episode_info.season
            episodes = episode_info.episodes
            reasons.append(f"Parsed episode: S{season:02d}E{episodes[0]:02d}")
            # Save episode title if found in filename (will be added to nfo_dict later)
            if episode_info.title:
                episode_title_from_filename = episode_info.title
                reasons.append(f"Episode title from filename: {episode_info.title}")
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
            # For filename-only TV episodes (no folder structure), extract year from filename
            year = extract_year(parent_folder) or extract_year(folder_name)
            if not year and not parent_folder:
                # Only extract year from filename if there's no folder structure (bare filename)
                # This avoids extracting episode dates as show years
                year = extract_year(filename)
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

        # Add episode title from filename if we found one and NFO didn't have one
        if episode_title_from_filename and not nfo_dict.get("episode_title"):
            nfo_dict["episode_title"] = episode_title_from_filename

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

        # Discover companion files
        sidecars = self._discover_companion_files(file_path)
        if sidecars:
            reasons.append(f"Found {len(sidecars)} companion files")

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
            sidecars=sidecars,
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
