"""Filename and folder patterns for media identification."""

import re
from typing import NamedTuple


class EpisodeMatch(NamedTuple):
    """Episode pattern match result."""

    season: int
    episodes: list[int]  # Can be multiple for ranges
    title: str | None = None


class MovieMatch(NamedTuple):
    """Movie pattern match result."""

    title: str
    year: int | None = None


class DateMatch(NamedTuple):
    """Date-based episode match result."""

    date: str  # YYYY-MM-DD format
    title: str | None = None


# Episode patterns (SxxEyy, 1x02, S01E01-E02, etc.)
EPISODE_PATTERNS = [
    # S01E02 or S01 E02 (allow separators) and optional range like -E03
    re.compile(
        r"[Ss](?P<season>\d{1,2})[ ._\-]*[Ee](?P<ep1>\d{1,3})(?:[ ._\-]*-?[ ._\-]*[Ee](?P<ep2>\d{1,3}))?",
        re.IGNORECASE,
    ),
    # 1x02 or 1x02-03 (require a dash to indicate a range; allow separators around dash)
    re.compile(
        r"(?P<season>\d{1,2})x(?P<ep1>\d{1,3})(?:[ ._\-]*-[ ._\-]*(?P<ep2>\d{1,3}))?",
        re.IGNORECASE,
    ),
]

# Date patterns (YYYY-MM-DD, YYYY.MM.DD, etc.)
DATE_PATTERN = re.compile(
    r"(?P<year>\d{4})[-.](?P<month>\d{2})[-.](?P<day>\d{2})",
)

# Year pattern for movies (1999, 2020, etc.)
YEAR_PATTERN = re.compile(r"\((?P<year>\d{4})\)|\b(?P<year2>19\d{2}|20\d{2})\b")

# Part pattern (Part 1, Part 2, pt1, etc.)
PART_PATTERN = re.compile(r"[Pp](?:ar)?t[.\s]*(?P<part>\d+)", re.IGNORECASE)

# Season folder pattern
SEASON_FOLDER_PATTERN = re.compile(
    r"[Ss]eason[.\s]*(?P<season>\d{1,2})",
    re.IGNORECASE,
)


def extract_episode_info(filename: str) -> EpisodeMatch | None:
    """
    Extract episode information from filename.

    Args:
        filename: Filename to parse

    Returns:
        EpisodeMatch if found, None otherwise
    """
    for pattern in EPISODE_PATTERNS:
        match = pattern.search(filename)
        if match:
            season = int(match.group("season"))
            ep1 = int(match.group("ep1"))
            episodes = [ep1]

            # Check for episode range
            ep2_str = match.groupdict().get("ep2")
            if ep2_str:
                episodes.append(int(ep2_str))

            return EpisodeMatch(season=season, episodes=episodes)

    return None


def extract_date(filename: str) -> DateMatch | None:
    """
    Extract date from filename (for daily shows).

    Args:
        filename: Filename to parse

    Returns:
        DateMatch if found, None otherwise
    """
    match = DATE_PATTERN.search(filename)
    if match:
        year = match.group("year")
        month = match.group("month")
        day = match.group("day")
        return DateMatch(date=f"{year}-{month}-{day}")

    return None


def extract_year(filename: str) -> int | None:
    """
    Extract year from filename.

    Args:
        filename: Filename to parse

    Returns:
        Year if found, None otherwise
    """
    match = YEAR_PATTERN.search(filename)
    if match:
        year_str = match.group("year") or match.group("year2")
        if year_str:
            return int(year_str)

    return None


def extract_part(filename: str) -> int | None:
    """
    Extract part number from filename (for multipart episodes/movies).

    Args:
        filename: Filename to parse

    Returns:
        Part number if found, None otherwise
    """
    match = PART_PATTERN.search(filename)
    if match:
        return int(match.group("part"))

    return None


def extract_season_from_folder(folder_name: str) -> int | None:
    """
    Extract season number from folder name.

    Args:
        folder_name: Folder name to parse

    Returns:
        Season number if found, None otherwise
    """
    match = SEASON_FOLDER_PATTERN.search(folder_name)
    if match:
        return int(match.group("season"))

    return None


def clean_title(title: str) -> str:
    """
    Clean up a title string by removing patterns and normalizing.

    Args:
        title: Raw title string

    Returns:
        Cleaned title
    """
    # Remove episode patterns
    for pattern in EPISODE_PATTERNS:
        title = pattern.sub("", title)

    # Remove date patterns
    title = DATE_PATTERN.sub("", title)

    # Remove year in parentheses
    title = re.sub(r"\(\d{4}\)", "", title)

    # Remove standalone year
    title = re.sub(r"\b(19\d{2}|20\d{2})\b", "", title)

    # Remove part indicators
    title = PART_PATTERN.sub("", title)

    # Remove common separators and clean up
    title = re.sub(r"[._\-]+", " ", title)

    # Remove quality indicators (1080p, 720p, etc.)
    title = re.sub(r"\b\d{3,4}[pi]\b", "", title, flags=re.IGNORECASE)

    # Remove codec info
    title = re.sub(
        r"\b(x264|x265|h264|h265|hevc|xvid|divx)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove common release/source tags
    title = re.sub(
        r"\b(webrip|web[- ]?dl|webdl|tvrip|bluray|bdrip|dvdrip|hdrip|hdtv|uhd|4k|amzn|nf)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove 'Seasons 1 to 5' or 'Season 1-5' expressions fully
    title = re.sub(r"\bseasons?\s*\d+\s*(?:to|-)+\s*\d+\b", "", title, flags=re.IGNORECASE)

    # Remove explicit 'Season 01' tokens entirely (word + number) early to avoid leaving stray numbers
    title = re.sub(r"\bseason[s]?\s*\d{1,2}\b", "", title, flags=re.IGNORECASE)

    # Remove collection words and numeric ranges (e.g., 'Complete Seasons 1 to 5')
    title = re.sub(r"\b(complete|seasons?|season pack)\b", "", title, flags=re.IGNORECASE)
    title = re.sub(r"\b\d+\s*(?:to|-)+\s*\d+\b", "", title, flags=re.IGNORECASE)
    # Remove leftover 'to 5' segments if previous removals split tokens
    title = re.sub(r"\bto\s*\d+\b", "", title, flags=re.IGNORECASE)

    # Unquote numeric-only tokens like '227'
    title = re.sub(r"'(?P<num>\d+)'", r"\g<num>", title)

    # Remove audio info
    title = re.sub(
        r"\b(aac|ac3|dts|truehd|atmos|dd5\.?1|dd2\.?0)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove release group tags [xxx]
    title = re.sub(r"\[.*?\]", "", title)

    # Collapse multiple spaces and trim
    title = re.sub(r"\s+", " ", title).strip()

    return title
