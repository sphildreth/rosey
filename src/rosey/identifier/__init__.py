"""Identifier module for media file identification."""

from rosey.identifier.identifier import Identifier, identify_file
from rosey.identifier.nfo import NFOData, find_nfo_for_file, parse_nfo
from rosey.identifier.patterns import (
    clean_title,
    extract_date,
    extract_episode_info,
    extract_part,
    extract_year,
)

__all__ = [
    "Identifier",
    "identify_file",
    "NFOData",
    "parse_nfo",
    "find_nfo_for_file",
    "clean_title",
    "extract_episode_info",
    "extract_date",
    "extract_year",
    "extract_part",
]
