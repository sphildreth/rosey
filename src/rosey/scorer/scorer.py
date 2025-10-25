"""Scoring engine for confidence calculations."""

import logging

from rosey.models import IdentificationResult, Score

logger = logging.getLogger(__name__)

# Confidence thresholds
GREEN_THRESHOLD = 70
YELLOW_THRESHOLD = 40


class Scorer:
    """Scores identification results with confidence levels."""

    def __init__(self, green_threshold: int = 70, yellow_threshold: int = 40):
        """
        Initialize scorer.

        Args:
            green_threshold: Minimum confidence for green level
            yellow_threshold: Minimum confidence for yellow level
        """
        self.green_threshold = green_threshold
        self.yellow_threshold = yellow_threshold

    def score(self, result: IdentificationResult) -> Score:
        """
        Calculate confidence score for an identification result.

        Args:
            result: IdentificationResult to score

        Returns:
            Score with confidence level and reasons
        """
        confidence = 0
        reasons = []

        item = result.item

        # Base score for identification type
        if item.kind == "unknown":
            confidence = 0
            reasons.append("Unknown media type")
            return Score(confidence=confidence, reasons=reasons)

        # NFO-based scoring (highest confidence)
        if item.nfo:
            if "imdbid" in item.nfo:
                confidence += 50
                reasons.append("IMDB ID from NFO")
            elif "tmdbid" in item.nfo:
                confidence += 45
                reasons.append("TMDB ID from NFO")
            elif "tvdbid" in item.nfo:
                confidence += 40
                reasons.append("TVDB ID from NFO")

        # Title confidence
        if item.title:
            if item.nfo and item.nfo.get("title"):
                confidence += 20
                reasons.append("Title from NFO")
            else:
                confidence += 10
                reasons.append("Title from filename")
        else:
            confidence -= 20
            reasons.append("No title identified")

        # Year/date confidence
        if item.kind == "movie":
            if item.year:
                confidence += 15
                reasons.append(f"Year identified: {item.year}")
            else:
                confidence -= 10
                reasons.append("No year found")

        # Episode-specific scoring
        if item.kind == "episode":
            if item.season is not None and item.episodes:
                confidence += 20
                reasons.append(f"Season/episode identified: S{item.season:02d}E{item.episodes[0]:02d}")
            elif item.date:
                confidence += 15
                reasons.append(f"Date episode identified: {item.date}")
            else:
                confidence -= 15
                reasons.append("No season/episode information")

            # Episode title bonus
            if item.nfo and item.nfo.get("episode_title"):
                confidence += 10
                reasons.append("Episode title from NFO")

        # Multipart handling
        if item.part:
            confidence += 5
            reasons.append(f"Part {item.part} identified")

        # Error penalties
        if result.errors:
            confidence -= 5 * len(result.errors)
            reasons.append(f"{len(result.errors)} error(s) during identification")

        # Clamp to 0-100 range
        confidence = max(0, min(100, confidence))

        return Score(confidence=confidence, reasons=reasons)

    def get_confidence_label(self, confidence: int) -> str:
        """
        Get confidence label (green/yellow/red).

        Args:
            confidence: Confidence score (0-100)

        Returns:
            Confidence label string
        """
        if confidence >= self.green_threshold:
            return "green"
        elif confidence >= self.yellow_threshold:
            return "yellow"
        else:
            return "red"


def score_identification(
    result: IdentificationResult,
    green_threshold: int = 70,
    yellow_threshold: int = 40,
) -> Score:
    """
    Convenience function to score an identification result.

    Args:
        result: IdentificationResult to score
        green_threshold: Minimum confidence for green level
        yellow_threshold: Minimum confidence for yellow level

    Returns:
        Score with confidence and reasons
    """
    scorer = Scorer(green_threshold=green_threshold, yellow_threshold=yellow_threshold)
    return scorer.score(result)
