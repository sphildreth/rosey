"""Media grouping logic after scanning.

Groups files by media directory and classifies them as movie, show, or unknown.
"""

import logging
import re
from pathlib import Path

from rosey.identifier.nfo import parse_nfo
from rosey.identifier.patterns import extract_episode_info, extract_season_from_folder
from rosey.mover.mover import SIDECAR_EXTENSIONS

logger = logging.getLogger(__name__)

# Generic organizational folders that don't qualify as media directories
GENERIC_ROOTS = {
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

# Permitted nested folders within a media directory
PERMITTED_NESTED = {
    "subs",
    "subtitles",
    "extras",
}


class MediaGroup:
    """Represents a media directory with its files."""

    def __init__(self, directory: str):
        """
        Initialize a media group.

        Args:
            directory: Absolute path to the media directory
        """
        self.directory = directory
        self.kind = "unknown"  # "movie" | "show" | "unknown"
        self.primary_videos: list[str] = []
        self.companions: dict[str, list[str]] = {}  # base_name -> list of companion files
        self.directory_companions: list[str] = []  # directory-level companions (e.g., movie.nfo)
        self.nfo_data: dict[str, str | int | None] = {}  # Parsed directory-level NFO
        self.errors: list[str] = []

    def __repr__(self) -> str:
        return f"MediaGroup(directory={self.directory}, kind={self.kind}, videos={len(self.primary_videos)})"


def get_media_directory(video_path: str, root_path: str) -> str:
    """
    Determine the media directory for a primary video file.

    The media directory is the nearest qualifying ancestor folder that is not
    a generic organizational root.

    Args:
        video_path: Absolute path to the video file
        root_path: The scan root path

    Returns:
        Absolute path to the media directory
    """
    video = Path(video_path)
    root = Path(root_path)

    # Start with the parent directory of the video
    current = video.parent

    # Walk up until we find a qualifying media directory or reach the root
    candidates = []
    while current != root and current != current.parent:
        folder_name = current.name.lower()

        # Check if this is a season folder
        if _is_season_folder(folder_name):
            # Season folders are nested inside the show folder
            # Return the parent as the media directory
            candidates.append(current.parent)
            break

        # Check if this is a generic root
        if folder_name not in GENERIC_ROOTS:
            candidates.append(current)

        current = current.parent

    # If we found candidates, return the first (closest to the video)
    if candidates:
        return str(candidates[0])

    # If no qualifying ancestor, use the parent directory of the video
    return str(video.parent)


def _is_season_folder(folder_name: str) -> bool:
    """Check if folder name indicates a season folder."""
    return extract_season_from_folder(folder_name) is not None or folder_name in PERMITTED_NESTED


def build_media_groups(
    video_files: list[str], root_path: str, enforce_one_media: bool = False
) -> list[MediaGroup]:
    """
    Build media groups from a list of video files.

    Args:
        video_files: List of absolute paths to video files
        root_path: The scan root path
        enforce_one_media: If True, flag mixed-content folders as errors

    Returns:
        List of MediaGroup objects
    """
    # Group videos by media directory
    groups_dict: dict[str, MediaGroup] = {}

    for video_path in video_files:
        media_dir = get_media_directory(video_path, root_path)

        if media_dir not in groups_dict:
            groups_dict[media_dir] = MediaGroup(media_dir)

        groups_dict[media_dir].primary_videos.append(video_path)

    # For each group, discover companions and parse NFO
    for group in groups_dict.values():
        _discover_companions(group)
        _parse_group_nfo(group)

    # Classify each group
    for group in groups_dict.values():
        classify_group(group, enforce_one_media)

    return list(groups_dict.values())


def _discover_companions(group: MediaGroup) -> None:
    """
    Discover companion files for a media group.

    Args:
        group: MediaGroup to populate with companions
    """
    media_dir = Path(group.directory)

    # Collect all files in the media directory and permitted nested folders
    all_files: list[Path] = []

    # Files directly in the media directory
    if media_dir.exists():
        all_files.extend([f for f in media_dir.iterdir() if f.is_file()])

        # Files in permitted nested folders
        for subfolder in media_dir.iterdir():
            if subfolder.is_dir():
                folder_name = subfolder.name.lower()
                # Include season folders and permitted nested folders
                if _is_season_folder(folder_name):
                    all_files.extend([f for f in subfolder.rglob("*") if f.is_file()])

    # Build a set of primary video base names
    primary_bases = {Path(v).stem for v in group.primary_videos}

    # Match companions to primary videos
    for file_path in all_files:
        if str(file_path) in group.primary_videos:
            continue  # Skip primary videos

        ext = file_path.suffix.lower()
        if ext not in SIDECAR_EXTENSIONS:
            continue  # Not a companion

        base_name = file_path.stem

        # Check if this companion matches a primary video
        matched = False
        for primary_base in primary_bases:
            if base_name == primary_base or base_name.startswith(primary_base + "."):
                if primary_base not in group.companions:
                    group.companions[primary_base] = []
                group.companions[primary_base].append(str(file_path))
                matched = True
                break

        # Directory-level companions (e.g., movie.nfo, poster.jpg)
        if not matched:
            common_names = {
                "movie",
                "tvshow",
                "show",
                "poster",
                "fanart",
                "banner",
                "landscape",
                "clearlogo",
            }
            if base_name.lower() in common_names:
                group.directory_companions.append(str(file_path))


def _parse_group_nfo(group: MediaGroup) -> None:
    """
    Parse directory-level NFO for a media group.

    Args:
        group: MediaGroup to populate with NFO data
    """
    media_dir = Path(group.directory)

    # Look for movie.nfo or tvshow.nfo
    for nfo_name in ["movie.nfo", "tvshow.nfo", "show.nfo"]:
        nfo_path = media_dir / nfo_name
        if nfo_path.exists():
            nfo_data = parse_nfo(str(nfo_path))
            if nfo_data:
                group.nfo_data = {
                    "title": nfo_data.title,
                    "year": nfo_data.year,
                    "tmdbid": nfo_data.tmdb_id,
                    "imdbid": nfo_data.imdb_id,
                    "tvdbid": nfo_data.tvdb_id,
                    "season": nfo_data.season,
                    "episode": nfo_data.episode,
                }
                logger.debug(f"Parsed NFO for {group.directory}: {nfo_name}")
                break


def classify_group(group: MediaGroup, enforce_one_media: bool = False) -> None:
    """
    Classify a media group as movie, show, or unknown.

    Args:
        group: MediaGroup to classify
        enforce_one_media: If True, flag mixed-content as errors
    """
    media_dir = Path(group.directory)

    # Check for season folders
    has_season_folders = any(
        _is_season_folder(child.name.lower()) for child in media_dir.iterdir() if child.is_dir()
    )

    # Check for episode patterns in filenames
    has_episode_patterns = any(
        extract_episode_info(Path(v).stem) is not None for v in group.primary_videos
    )

    # Check for date-based episode patterns
    has_date_patterns = any(_has_date_pattern(Path(v).stem) for v in group.primary_videos)

    # Check NFO for season/episode info
    nfo_has_season = group.nfo_data.get("season") is not None

    # Classification rules (in order of precedence)
    if has_season_folders or has_episode_patterns or has_date_patterns or nfo_has_season:
        group.kind = "show"
    elif len(group.primary_videos) == 1:
        # Single video file suggests movie
        if group.nfo_data.get("tmdbid") or group.nfo_data.get("imdbid"):
            # NFO with TMDB/IMDB ID and no season -> movie
            if not nfo_has_season:
                group.kind = "movie"
            else:
                group.kind = "show"
        else:
            # Default single video to movie
            group.kind = "movie"
    elif len(group.primary_videos) > 1:
        # Multiple videos without clear TV markers
        if enforce_one_media:
            group.kind = "unknown"
            group.errors.append(
                f"Mixed content: {len(group.primary_videos)} videos without clear classification"
            )
        else:
            # Heuristic: check if all videos share a common prefix (likely a show)
            common_prefix = _find_common_prefix([Path(v).stem for v in group.primary_videos])
            if common_prefix and len(common_prefix) > 3:
                group.kind = "show"
            else:
                group.kind = "unknown"
    else:
        group.kind = "unknown"

    logger.debug(f"Classified {group.directory} as {group.kind}")


def _has_date_pattern(filename: str) -> bool:
    """Check if filename contains a date pattern (YYYY-MM-DD)."""
    date_pattern = re.compile(r"\d{4}-\d{2}-\d{2}")
    return bool(date_pattern.search(filename))


def _find_common_prefix(names: list[str]) -> str:
    """Find the longest common prefix among a list of names."""
    if not names:
        return ""

    # Start with the first name
    prefix = names[0]

    for name in names[1:]:
        # Find common prefix between current prefix and this name
        i = 0
        while i < len(prefix) and i < len(name) and prefix[i] == name[i]:
            i += 1
        prefix = prefix[:i]

        if not prefix:
            break

    return prefix.strip()
