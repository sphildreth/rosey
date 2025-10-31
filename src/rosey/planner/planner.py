"""Path planning for Jellyfin-compatible media organization."""

import logging
import re
from pathlib import Path

from rosey.models import MediaItem

logger = logging.getLogger(__name__)


def title_case(text: str) -> str:
    """
    Title case a string, but don't capitalize common articles and prepositions.

    Args:
        text: Text to title case

    Returns:
        Title cased text
    """
    # Words that should not be capitalized (except at start)
    lowercase_words = {
        "a",
        "an",
        "and",
        "as",
        "at",
        "but",
        "by",
        "for",
        "if",
        "in",
        "nor",
        "of",
        "on",
        "or",
        "so",
        "the",
        "to",
        "up",
        "yet",
    }

    words = text.split()
    if not words:
        return text

    # Capitalize first word
    result = [words[0].capitalize()]

    # Capitalize other words unless they're in lowercase_words
    for word in words[1:]:
        if word.lower() in lowercase_words:
            result.append(word.lower())
        else:
            result.append(word.capitalize())

    return " ".join(result)


# Invalid filename characters for Windows (also good for Linux)
INVALID_CHARS = r'<>:"/\|?*'

# Windows reserved names
WINDOWS_RESERVED = {
    "CON",
    "PRN",
    "AUX",
    "NUL",
    "COM1",
    "COM2",
    "COM3",
    "COM4",
    "COM5",
    "COM6",
    "COM7",
    "COM8",
    "COM9",
    "LPT1",
    "LPT2",
    "LPT3",
    "LPT4",
    "LPT5",
    "LPT6",
    "LPT7",
    "LPT8",
    "LPT9",
}


class Planner:
    """Plans destination paths for media files."""

    def __init__(self, movies_root: str = "", tv_root: str = ""):
        """
        Initialize planner.

        Args:
            movies_root: Root directory for movies
            tv_root: Root directory for TV shows
        """
        self.movies_root = movies_root
        self.tv_root = tv_root

    def plan_destination(self, item: MediaItem) -> str:
        """
        Plan destination path for a media item.

        Args:
            item: MediaItem to plan for

        Returns:
            Destination path string
        """
        if item.kind == "movie":
            return self._plan_movie(item)
        elif item.kind == "episode":
            return self._plan_episode(item)
        else:
            # Unknown - return original path
            return item.source_path

    def _plan_movie(self, item: MediaItem) -> str:
        """Plan path for a movie."""
        if not self.movies_root:
            return item.source_path

        # Build movie folder name: "Movie Title (Year)" or "Movie Title (Year) [tmdbid-ID]"
        title = sanitize_name(item.title or "Unknown")

        base_name = f"{title} ({item.year})" if item.year else title

        # Add TMDB ID if available
        if item.nfo and item.nfo.get("tmdbid"):
            tmdbid = item.nfo["tmdbid"]
            folder_name = f"{base_name} [tmdbid-{tmdbid}]"
        else:
            folder_name = base_name

        # Build filename
        ext = Path(item.source_path).suffix

        filename = f"{folder_name} Part {item.part}{ext}" if item.part else f"{folder_name}{ext}"

        filename = sanitize_name(filename)

        # Full path: movies_root/Movie Title (Year) [tmdbid-ID]/Movie Title (Year) [tmdbid-ID].ext
        return str(Path(self.movies_root) / folder_name / filename)

    def _plan_episode(self, item: MediaItem) -> str:
        """Plan path for a TV episode."""
        if not self.tv_root:
            return item.source_path

        # Show title with optional year
        title = title_case(item.title or "Unknown Show")
        title = sanitize_name(title)
        show_folder = f"{title} ({item.year})" if item.year else title

        # Add TMDB ID if available
        if item.nfo and item.nfo.get("tmdbid"):
            tmdbid = item.nfo["tmdbid"]
            show_folder = f"{show_folder} [tmdbid-{tmdbid}]"

        # Season folder (specials go to Season 00)
        season_num = item.season if item.season is not None else 0
        season_folder = f"Season {season_num:02d}"

        # Build filename
        ext = Path(item.source_path).suffix

        if item.date:
            # Date-based episode: "Show - 2020-01-15.ext"
            filename = f"{title} - {item.date}{ext}"
        elif item.episodes:
            # Episode-based
            if len(item.episodes) == 1:
                ep_str = f"S{season_num:02d}E{item.episodes[0]:02d}"
            else:
                # Multi-episode: "S01E01-E02"
                ep_str = f"S{season_num:02d}E{item.episodes[0]:02d}-E{item.episodes[-1]:02d}"

            if item.part:
                # Multipart same episode: "Show - S01E01 Part 1.ext"
                filename = f"{title} - {ep_str} Part {item.part}{ext}"
            else:
                # Standard: "Show - S01E01.ext" or with episode title
                episode_title = item.nfo.get("episode_title") if item.nfo else None
                if episode_title:
                    episode_title = sanitize_name(episode_title)
                    filename = f"{title} - {ep_str} - {episode_title}{ext}"
                else:
                    filename = f"{title} - {ep_str}{ext}"
        else:
            # No episode info - just use show name
            filename = f"{title}{ext}"

        filename = sanitize_name(filename)

        # Full path: tv_root/Show (Year) [tmdbid-ID]/Season NN/Show - S01E01.ext
        return str(Path(self.tv_root) / show_folder / season_folder / filename)


def sanitize_name(name: str) -> str:
    """
    Sanitize a filename or folder name for cross-platform compatibility.

    Args:
        name: Name to sanitize

    Returns:
        Sanitized name
    """
    # Remove invalid characters
    for char in INVALID_CHARS:
        name = name.replace(char, "")

    # Replace multiple spaces with single space
    name = re.sub(r"\s+", " ", name)

    # Remove leading/trailing spaces and dots
    name = name.strip(" .")

    # Check for Windows reserved names
    base_name = name.split(".")[0].upper()
    if base_name in WINDOWS_RESERVED:
        name = f"{name}_media"

    # Ensure not empty
    if not name:
        name = "unknown"

    return name


def plan_path(
    item: MediaItem,
    movies_root: str = "",
    tv_root: str = "",
) -> str:
    """
    Convenience function to plan a destination path.

    Args:
        item: MediaItem to plan for
        movies_root: Root directory for movies
        tv_root: Root directory for TV shows

    Returns:
        Destination path
    """
    planner = Planner(movies_root=movies_root, tv_root=tv_root)
    return planner.plan_destination(item)
