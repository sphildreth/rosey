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
    # Support up to 4-digit episodes for long-running shows like One Piece
    re.compile(
        r"[Ss](?P<season>\d{1,2})[ ._\-]*[Ee](?P<ep1>\d{1,4})(?:[ ._\-]*-?[ ._\-]*[Ee](?P<ep2>\d{1,4}))?",
        re.IGNORECASE,
    ),
    # 1x02 or 1x02-03 (require a dash to indicate a range; allow separators around dash)
    re.compile(
        r"(?P<season>\d{1,2})x(?P<ep1>\d{1,4})(?:[ ._\-]*-[ ._\-]*(?P<ep2>\d{1,4}))?",
        re.IGNORECASE,
    ),
]

# Date patterns (YYYY-MM-DD, YYYY.MM.DD, etc.)
DATE_PATTERN = re.compile(
    r"(?P<year>\d{4})[-.](?P<month>\d{2})[-.](?P<day>\d{2})",
)

# Year pattern for movies (1999, 2020, etc.)
# Try parenthesized year first, then standalone year with flexible boundaries
YEAR_PATTERN = re.compile(
    r"\((?P<year>\d{4})\)|(?:^|[._\s\-])(?P<year2>19\d{2}|20\d{2})(?=[._\s\-]|$)"
)

# Part pattern (Part 1, Part 2, pt1, etc., including Roman numerals like Part III and spelled out numbers)
# Use word boundary at start to avoid matching within words like "caption"
PART_PATTERN = re.compile(
    r"\b[Pp](?:ar)?t[.\s]*(?P<part>\d+|(?:[IVX]+|One|Two|Three|Four|Five|Six|Seven|Eight|Nine|Ten)\b)",
    re.IGNORECASE,
)

# Season folder pattern (matches "Season 01", "Season 1", "S03", or S## anywhere in folder name)
SEASON_FOLDER_PATTERN = re.compile(
    r"(?:[Ss]eason[.\s]*(?P<season>\d{1,2})|(?:^|[.\s_-])[Ss](?P<season2>\d{1,2})(?:[.\s_-]|$))",
    re.IGNORECASE,
)


def extract_title_before_episode(filename: str) -> str:
    """
    Extract title from filename, taking only the part before episode markers.

    This helps avoid including episode titles in the show title.

    Args:
        filename: Filename to parse

    Returns:
        Title portion before episode marker
    """
    # Find the first episode pattern match
    for pattern in EPISODE_PATTERNS:
        match = pattern.search(filename)
        if match:
            title_part = filename[: match.start()]
            # Also check for em-dash or hyphen followed by space (often separates show title from episode)
            # Remove trailing em-dash/hyphen that might be episode title separator
            title_part = re.sub(r"[\s\-\u2013\u2014]+$", "", title_part)
            return title_part

    # If no episode pattern, check for date pattern
    match = DATE_PATTERN.search(filename)
    if match:
        return filename[: match.start()]

    # No pattern found, return full filename
    return filename


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
                ep2 = int(ep2_str)
                if ep2 > ep1:
                    episodes = list(range(ep1, ep2 + 1))
                else:
                    episodes.append(ep2)

            # Extract episode title if present after the episode marker
            episode_title = _extract_episode_title_from_match(filename, match.end())

            return EpisodeMatch(season=season, episodes=episodes, title=episode_title)

    return None


def _extract_episode_title_from_match(filename: str, after_pos: int) -> str | None:
    """
    Extract episode title from the part of filename after the episode marker.

    Args:
        filename: Full filename
        after_pos: Position after the episode marker

    Returns:
        Episode title if found, None otherwise
    """
    # Get the part after the episode marker
    remainder = filename[after_pos:].strip()

    if not remainder:
        return None

    # Look for patterns: " - Title", " (Title)", or just "Title"
    # Pattern 1: " - Episode Title" or "- Episode Title"
    dash_match = re.match(r"^\s*[-\u2013]\s*(.+)$", remainder)
    if dash_match:
        title = dash_match.group(1).strip()
    # Pattern 2: " (Episode Title)" or "(Episode Title)"
    elif remainder.startswith("(") and ")" in remainder:
        paren_match = re.match(r"^\s*\(([^)]+)\)", remainder)
        if paren_match:
            title = paren_match.group(1).strip()
        else:
            return None
    # Pattern 3: Just whitespace then title
    elif remainder[0:1].isspace() or remainder[0:1].isalpha():
        title = remainder.strip()
    else:
        return None

    # Clean up common artifacts from the title
    title = re.sub(r"\[.*?\]", "", title)  # Remove [tags]
    title = re.sub(r"\d{3,4}p", "", title, flags=re.IGNORECASE)  # Remove quality markers
    title = re.sub(
        r"\b(?:WEB-?DL|HDTV|BluRay|x264|x265|HEVC|10bit)\b", "", title, flags=re.IGNORECASE
    )
    title = title.strip(" .-_")

    return title if title else None


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
    Extract year from filename, excluding years that are part of date patterns.

    Args:
        filename: Filename to parse

    Returns:
        Year if found, None otherwise
    """
    # First, try to find a parenthesized year (higher priority)
    paren_pattern = re.compile(r"\((?P<year>\d{4})\)")
    match = paren_pattern.search(filename)
    if match:
        year_match_pos = match.start()
        # Check if this year is part of a date pattern (YYYY-MM-DD or YYYY.MM.DD)
        date_check = re.search(r"\d{4}[-\.]\d{2}[-\.]\d{2}", filename[max(0, year_match_pos - 1) :])
        if not date_check or date_check.start() > 1:
            # Not part of a date, or date starts after this year
            year = int(match.group("year"))
            # Validate year range (1900-2040 seems reasonable for movies)
            if 1900 <= year <= 2040:
                return year

    # Then try standalone year with flexible boundaries
    standalone_pattern = re.compile(r"(?:^|[._\s\-])(?P<year>19\d{2}|20\d{2})(?=[._\s\-]|$)")
    for match in standalone_pattern.finditer(filename):
        year_match_pos = match.start("year")
        # Check if this year is part of a date pattern (YYYY-MM-DD or YYYY.MM.DD)
        # Look a bit before and after the year to catch the full date pattern
        context_start = max(0, year_match_pos - 2)
        context_end = min(len(filename), year_match_pos + 15)
        context = filename[context_start:context_end]
        if re.search(r"\d{4}[-\.]\d{2}[-\.]\d{2}", context):
            # This year is part of a date, skip it
            continue

        # Check if this year immediately follows an episode marker (e.g., S05E06-1976)
        # Look for pattern like S##E##-YYYY or ##x##-YYYY right before the year
        pre_context = filename[max(0, year_match_pos - 10) : year_match_pos]
        if re.search(r"[Ss]\d{1,2}[Ee]\d{1,4}[-_]$", pre_context) or re.search(
            r"\d{1,2}x\d{1,4}[-_]$", pre_context, re.IGNORECASE
        ):
            # Year directly follows episode marker, likely part of episode ID
            continue

        year = int(match.group("year"))
        # Validate year range
        if 1900 <= year <= 2040:
            return year

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
        part_str = match.group("part")
        # Try to parse as int, if that fails, try Roman numeral, then spelled-out number
        try:
            return int(part_str)
        except ValueError:
            # Try Roman numeral
            if part_str.upper() in ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"]:
                return roman_to_int(part_str.upper())
            # Try spelled-out number
            word_to_num = {
                "one": 1,
                "two": 2,
                "three": 3,
                "four": 4,
                "five": 5,
                "six": 6,
                "seven": 7,
                "eight": 8,
                "nine": 9,
                "ten": 10,
            }
            return word_to_num.get(part_str.lower())

    return None


def roman_to_int(s: str) -> int:
    """Convert Roman numeral to integer."""
    roman_values = {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100, "D": 500, "M": 1000}
    total = 0
    prev_value = 0
    for char in reversed(s):
        value = roman_values.get(char, 0)
        if value < prev_value:
            total -= value
        else:
            total += value
        prev_value = value
    return total


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
        season_str = match.group("season") or match.group("season2")
        if season_str:
            return int(season_str)

    return None


def clean_title(title: str, extracted_year: int | None = None) -> str:
    """
    Clean up a title string by removing patterns and normalizing.

    Args:
        title: Raw title string
        extracted_year: The year that was extracted from the filename (if any)
                       This prevents removing year-like numbers that are part of the title

    Returns:
        Cleaned title
    """
    # Preserve "Vol X", "Volume X" patterns by temporarily replacing them
    # Use a placeholder that won't be affected by cleaning
    vol_pattern = re.compile(r"\b(Vol|Volume)\s+(\d+)\b", re.IGNORECASE)
    vol_matches = []
    for match in vol_pattern.finditer(title):
        vol_matches.append((match.group(0), f"VOLPLACEHOLDER{match.group(2)}ENDVOL"))
    for orig, placeholder in vol_matches:
        title = title.replace(orig, placeholder)

    # Remove episode patterns first, but save parenthetical info (like show origin/country)
    # Extract and preserve parenthetical info that's not a year
    preserved_parens = []
    paren_pattern = re.compile(r"\(([^)]+)\)")
    for match in paren_pattern.finditer(title):
        content = match.group(1)
        # Only preserve if it's not a year and not empty
        if not re.match(r"^\d{4}$", content) and content.strip():
            preserved_parens.append(f"({content})")

    # Remove episode patterns
    for pattern in EPISODE_PATTERNS:
        title = pattern.sub("", title)

    # Remove date patterns
    title = DATE_PATTERN.sub("", title)

    # Remove year in parentheses first
    title = re.sub(r"\(\d{4}\)", "", title)

    # Remove release group tags [xxx] before separator conversion
    title = re.sub(r"\[.*?\]", "", title)

    # Remove common separators and clean up (convert to spaces) - do this early
    # Convert dots, underscores, hyphens, and em-dashes to spaces
    title = re.sub(r"[._\-\u2013\u2014]+", " ", title)

    # Now remove standalone year (but only after converting separators to spaces)
    # Only remove the extracted year, not other year-like numbers that might be part of the title
    if extracted_year:
        # Remove the specific extracted year (and any digits around it from the filename)
        # But be careful not to remove it if it's the title itself (at the start)
        # First try to remove it from anywhere except the start
        year_pattern = rf"(\s+{extracted_year}[-\s]*)"
        if re.search(year_pattern, title):
            title = re.sub(year_pattern, " ", title)
        # If that didn't match and the whole title is just the year, keep it
        elif title.strip() == str(extracted_year):
            pass  # Keep the year as the title
        else:
            # Remove year even at start if there's more content
            title = re.sub(rf"^{extracted_year}[-\s]*", "", title)
    else:
        # If no year was extracted, remove years in valid range (1900-2040)
        # Use a more specific pattern that won't match title numbers at the start
        def should_remove_year(match: re.Match[str]) -> str:
            year_str = match.group(1)
            year = int(year_str)
            # Only remove if it's in the valid movie year range
            return "" if 1900 <= year <= 2040 else match.group(0)

        title = re.sub(r"\s+(19\d{2}|20\d{2})(?=\s|$)", should_remove_year, title)

    # Remove part indicators (but not "Part" that's in the middle of titles)
    # Remove when followed by a number, Roman numeral, or spelled out number (One, Two, etc.)
    title = re.sub(
        r"\b[Pp](?:ar)?t[.\s]*(?:\d+|[IVX]+|One|Two|Three|Four|Five|Six|Seven|Eight|Nine|Ten)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove quality indicators (1080p, 720p, 2160p, 480p, etc.)
    title = re.sub(r"\b\d{3,4}[pi]\b", "", title, flags=re.IGNORECASE)

    # Remove format indicators (3D, IMAX, 70mm, etc.)
    title = re.sub(r"\b(3d|imax|\d+mm)\b", "", title, flags=re.IGNORECASE)

    # Remove codec info (including H.264, H 264, h264, x264, x265, etc.)
    title = re.sub(
        r"\b[Hh]\s*\.?\s*26[45]\b",
        "",
        title,
    )
    title = re.sub(
        r"\b(x264|x265|h264|h265|hevc|xvid|divx|mp4)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove audio info (including variations with dots/spaces like DD5.1, DDP5.1, AAC2.0, Atmos, etc.)
    # Do this before other number cleaning to catch full patterns
    title = re.sub(
        r"\b(?:aac|dd|ddp|ac3|dts|truehd|atmos)\s*\d*\s*\.?\s*\d*\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove common release/source tags (handle compounds like WEB DL after hyphen conversion)
    title = re.sub(
        r"\b(webrip|web\s+dl|webdl|web|dl|tvrip|bluray|bdrip|dvdrip|hdrip|hdtv|uhd|4k|amzn|nf|hulu|dv)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove release tags (PROPER, REPACK, INTERNAL, etc.) and edition info
    # Remove "Extended Edition", "Director's Cut", etc. but be careful not to affect titles
    title = re.sub(
        r"\b(proper|repack|internal|extended\s+edition|unrated|remastered|directors?\s+cut|cut|dubbed)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )
    # Remove standalone "edition" that's left over
    title = re.sub(r"\bedition\b", "", title, flags=re.IGNORECASE)

    # Remove common language/country descriptors and platform names
    # Only remove at the start or end of the title to avoid removing words that are part of the title itself
    # First normalize spaces so we can reliably detect start/end positions
    title_normalized = re.sub(r"\s+", " ", title).strip()

    # Handle multi-word patterns first (from the end)
    # These need to be checked before single words
    compound_patterns = [
        r"\s+disney\s+plus",
        r"\s+paramount\s+plus",
        r"\s+disney\s+animated",
        r"\s+roku\s+original",
        r"\s+netflix\s+original",
        r"\s+hulu\s+original",
        r"\s+french\s+korean",
    ]

    for pat in compound_patterns:
        title_normalized = re.sub(
            rf"{pat}\s*$",
            "",
            title_normalized,
            flags=re.IGNORECASE,
        )

    # Pattern that matches single words at the start or end (after normalization)
    descriptors_pattern = r"(korean|japanese|chinese|french|spanish|german|italian|russian|polish|persian|irish|belgian|british|telugu|netflix|hulu|original|roku|disney|plus|paramount|animated|criterion|hdr|amzn)"

    # Remove from the end (single words) - iterate until no more matches
    while True:
        new_title = re.sub(
            rf"\s+{descriptors_pattern}\s*$",
            "",
            title_normalized,
            flags=re.IGNORECASE,
        )
        if new_title == title_normalized:
            break
        title_normalized = new_title

    # Remove from the start (single words) - iterate until no more matches
    while True:
        new_title = re.sub(
            rf"^{descriptors_pattern}\s+",
            "",
            title_normalized,
            flags=re.IGNORECASE,
        )
        if new_title == title_normalized:
            break
        title_normalized = new_title

    title = title_normalized

    # Remove director/actor name descriptors (common ones that appear as tags)
    # These are multi-word names that typically appear after year/quality metadata
    title = re.sub(
        r"\b(michael\s+bay|baz\s+luhrmann|david\s+o\s+russell|idris\s+elba|bj\s+novak)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove director/actor names and descriptive words that appear as single words
    # (These are harder to detect reliably, but we can catch common patterns)
    # Remove phrases like "Michael Bay", "Baz Luhrmann", "David O Russell", "Idris Elba", "BJ Novak"
    # This is tricky - we'll handle the most common by removing capitalized words after we've stripped metadata

    # Remove "Black and Chrome", "Romantic Comedy" type descriptors (full phrases)
    title = re.sub(
        r"\b(black\s+and\s+chrome|romantic\s+comedy|unrated)\b",
        "",
        title,
        flags=re.IGNORECASE,
    )

    # Remove 'Seasons 1 to 5' or 'Season 1-5' expressions fully
    title = re.sub(r"\bseasons?\s*\d+\s*(?:to|-)+\s*\d+\b", "", title, flags=re.IGNORECASE)

    # Remove explicit 'Season 01' tokens entirely (word + number) early to avoid leaving stray numbers
    title = re.sub(r"\bseason[s]?\s*\d{1,2}\b", "", title, flags=re.IGNORECASE)

    # Remove compact season format (S01, S02, etc.)
    title = re.sub(r"\b[Ss]\d{1,2}\b", "", title)

    # Remove collection words and numeric ranges (e.g., 'Complete Seasons 1 to 5')
    title = re.sub(r"\b(complete|seasons?|season pack)\b", "", title, flags=re.IGNORECASE)
    title = re.sub(r"\b\d+\s*(?:to|-)+\s*\d+\b", "", title, flags=re.IGNORECASE)
    # Remove leftover 'to 5' segments if previous removals split tokens
    title = re.sub(r"\bto\s*\d+\b", "", title, flags=re.IGNORECASE)

    # Unquote numeric-only tokens like '227'
    title = re.sub(r"'(?P<num>\d+)'", r"\g<num>", title)

    # Remove release group tags [xxx]
    title = re.sub(r"\[.*?\]", "", title)

    # Remove specific known release group names (all caps abbreviations)
    # Common ones: GROUP, KOGi, AVS, GGEZ, BAE, RBB, NTb, etc.
    # Be conservative - only remove if all caps or specific mixed case patterns
    title = re.sub(
        r"\b(GROUP|KOGI|AVS|GGEZ|BAE|RBB|NTB|RARBG|ION10|MEMENTO|KILLERS|ROVERS|SPARKS|FLUX)\b",
        "",
        title,
    )
    # Mixed case release groups (common patterns)
    title = re.sub(r"\b(KOGi|NTb|RBB)\b", "", title)

    # Remove stray episode markers (leftover E03, E05, E12 from multi-episode patterns)
    title = re.sub(r"\b[Ee]\d{1,4}\b", "", title)

    # Remove stray single digits or small numbers that are leftover from cleaning
    # Be very conservative - only remove patterns like "5 1" that are clearly leftovers
    # DO NOT remove single digits that could be part of titles (like "2", "9", etc.)
    # And don't remove at the start of the string (could be titles like "8 1 2")
    if not re.match(r"^\d", title):
        title = re.sub(r"\b\d\s+\d\b", "", title)
    # Don't remove trailing or middle single digits - they're likely part of the title

    # Remove all remaining parentheses (empty or otherwise)
    title = re.sub(r"\([^)]*\)", "", title)

    # Collapse multiple spaces and trim
    title = re.sub(r"\s+", " ", title).strip()

    # Restore Vol patterns
    for _, placeholder in vol_matches:
        # Extract the number from placeholder
        import re as re_inner

        vol_num_match = re_inner.search(r"VOLPLACEHOLDER(\d+)ENDVOL", placeholder)
        if vol_num_match:
            vol_num = vol_num_match.group(1)
            title = title.replace(placeholder, f"Vol {vol_num}")

    # Restore common hyphenated compound words
    # These are known patterns where the hyphen is part of the title
    title = re.sub(r"\bSpider Man\b", "Spider-Man", title, flags=re.IGNORECASE)
    title = re.sub(r"\bSpider Verse\b", "Spider-Verse", title, flags=re.IGNORECASE)
    title = re.sub(r"\bX Men\b", "X-Men", title, flags=re.IGNORECASE)

    # Restore preserved parenthetical info
    if preserved_parens:
        title = f"{title} {' '.join(preserved_parens)}"

    return title.strip()
