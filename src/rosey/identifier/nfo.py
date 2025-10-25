"""NFO file parser for media metadata."""

import contextlib
import logging
import xml.etree.ElementTree as ET
from pathlib import Path

logger = logging.getLogger(__name__)


class NFOData:
    """Parsed NFO data."""

    def __init__(
        self,
        title: str | None = None,
        year: int | None = None,
        imdb_id: str | None = None,
        tmdb_id: str | None = None,
        tvdb_id: str | None = None,
        episode_title: str | None = None,
        season: int | None = None,
        episode: int | None = None,
    ):
        self.title = title
        self.year = year
        self.imdb_id = imdb_id
        self.tmdb_id = tmdb_id
        self.tvdb_id = tvdb_id
        self.episode_title = episode_title
        self.season = season
        self.episode = episode


def parse_nfo(nfo_path: str) -> NFOData | None:
    """
    Parse an NFO file.

    Args:
        nfo_path: Path to NFO file

    Returns:
        NFOData if successfully parsed, None otherwise
    """
    try:
        tree = ET.parse(nfo_path)
        root = tree.getroot()

        data = NFOData()

        # Extract common fields
        title_elem = root.find("title")
        if title_elem is not None and title_elem.text:
            data.title = title_elem.text.strip()

        year_elem = root.find("year")
        if year_elem is not None and year_elem.text:
            with contextlib.suppress(ValueError):
                data.year = int(year_elem.text.strip())

        # Extract IDs (direct tags)
        # IMDB
        imdb_elem = root.find(".//imdbid")
        if imdb_elem is None:
            imdb_elem = root.find(".//imdb_id")
        if imdb_elem is not None and imdb_elem.text:
            data.imdb_id = normalize_imdb_id(imdb_elem.text.strip())

        # TMDB
        tmdb_elem = root.find(".//tmdbid")
        if tmdb_elem is None:
            tmdb_elem = root.find(".//tmdb_id")
        if tmdb_elem is not None and tmdb_elem.text:
            data.tmdb_id = tmdb_elem.text.strip()

        # TVDB
        tvdb_elem = root.find(".//tvdbid")
        if tvdb_elem is None:
            tvdb_elem = root.find(".//tvdb_id")
        if tvdb_elem is not None and tvdb_elem.text:
            data.tvdb_id = tvdb_elem.text.strip()

        # Extract IDs via <uniqueid type="...">VALUE</uniqueid> (Kodi convention)
        try:
            for uid in root.findall(".//uniqueid"):
                type_attr = (uid.get("type") or "").lower()
                val = (uid.text or "").strip()
                if not val:
                    continue
                if type_attr == "imdb" and not data.imdb_id:
                    data.imdb_id = normalize_imdb_id(val)
                elif type_attr == "tmdb" and not data.tmdb_id:
                    data.tmdb_id = val
                elif type_attr == "tvdb" and not data.tvdb_id:
                    data.tvdb_id = val
        except Exception:
            # Be resilient to odd XML structures
            pass

        # Episode-specific fields
        ep_title_elem = root.find("episodetitle")
        if ep_title_elem is None:
            ep_title_elem = root.find("episode_title")
        if ep_title_elem is not None and ep_title_elem.text:
            data.episode_title = ep_title_elem.text.strip()

        season_elem = root.find("season")
        if season_elem is not None and season_elem.text:
            with contextlib.suppress(ValueError):
                data.season = int(season_elem.text.strip())

        episode_elem = root.find("episode")
        if episode_elem is not None and episode_elem.text:
            with contextlib.suppress(ValueError):
                data.episode = int(episode_elem.text.strip())

        return data

    except ET.ParseError as e:
        logger.warning(f"Failed to parse NFO {nfo_path}: {e}")
        return None
    except Exception as e:
        logger.error(f"Error reading NFO {nfo_path}: {e}")
        return None


def find_nfo_for_file(video_path: str) -> str | None:
    """
    Find an NFO file associated with a video file.

    Looks for:
    - Same name as video file with .nfo extension
    - movie.nfo or tvshow.nfo in same directory

    Args:
        video_path: Path to video file

    Returns:
        Path to NFO file if found, None otherwise
    """
    video = Path(video_path)
    video_dir = video.parent

    # Try same-name NFO
    nfo_path = video.with_suffix(".nfo")
    if nfo_path.exists():
        return str(nfo_path)

    # Try movie.nfo or tvshow.nfo
    for name in ["movie.nfo", "tvshow.nfo"]:
        nfo_path = video_dir / name
        if nfo_path.exists():
            return str(nfo_path)

    return None


def normalize_imdb_id(imdb_id: str) -> str:
    """
    Normalize IMDB ID to tt1234567 format.

    Args:
        imdb_id: Raw IMDB ID

    Returns:
        Normalized IMDB ID
    """
    # Remove any URL components
    if "imdb.com" in imdb_id:
        parts = imdb_id.split("/")
        for part in parts:
            if part.startswith("tt"):
                imdb_id = part

    # Ensure tt prefix
    if not imdb_id.startswith("tt"):
        imdb_id = f"tt{imdb_id}"

    return imdb_id
