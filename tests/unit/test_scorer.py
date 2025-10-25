"""Tests for scorer module."""


from rosey.identifier import identify_file
from rosey.models import IdentificationResult, MediaItem
from rosey.scorer import Scorer, score_identification


class TestScorerBasic:
    """Test basic scoring functionality."""

    def test_unknown_gets_zero(self):
        """Test that unknown media gets 0 confidence."""
        item = MediaItem(kind="unknown", source_path="/path/file.mkv")
        result = IdentificationResult(item=item, reasons=[])

        scorer = Scorer()
        score = scorer.score(result)

        assert score.confidence == 0

    def test_movie_with_nfo_ids(self):
        """Test movie with NFO IDs gets high confidence."""
        item = MediaItem(
            kind="movie",
            source_path="/path/Matrix.mkv",
            title="The Matrix",
            year=1999,
            nfo={"imdbid": "tt0133093"},
        )
        result = IdentificationResult(item=item, reasons=["NFO with IMDB ID"])

        scorer = Scorer()
        score = scorer.score(result)

        # Should have high confidence: IMDB ID (50) + title from NFO (20) + year (15)
        assert score.confidence >= 70

    def test_episode_with_season_episode(self):
        """Test episode with season/episode info."""
        item = MediaItem(
            kind="episode",
            source_path="/path/Show.S01E05.mkv",
            title="Show",
            season=1,
            episodes=[5],
        )
        result = IdentificationResult(item=item, reasons=["Parsed S01E05"])

        scorer = Scorer()
        score = scorer.score(result)

        # Should have decent confidence: title (10) + season/episode (20)
        assert score.confidence >= 30


class TestScorerThresholds:
    """Test confidence threshold logic."""

    def test_green_threshold(self):
        """Test green threshold (>=70)."""
        scorer = Scorer(green_threshold=70)

        assert scorer.get_confidence_label(70) == "green"
        assert scorer.get_confidence_label(85) == "green"
        assert scorer.get_confidence_label(100) == "green"

    def test_yellow_threshold(self):
        """Test yellow threshold (40-69)."""
        scorer = Scorer(yellow_threshold=40)

        assert scorer.get_confidence_label(40) == "yellow"
        assert scorer.get_confidence_label(55) == "yellow"
        assert scorer.get_confidence_label(69) == "yellow"

    def test_red_threshold(self):
        """Test red threshold (<40)."""
        scorer = Scorer(yellow_threshold=40)

        assert scorer.get_confidence_label(0) == "red"
        assert scorer.get_confidence_label(20) == "red"
        assert scorer.get_confidence_label(39) == "red"


class TestScorerFactors:
    """Test individual scoring factors."""

    def test_nfo_imdb_bonus(self):
        """Test IMDB ID bonus."""
        item = MediaItem(
            kind="movie",
            source_path="/path/file.mkv",
            title="Movie",
            nfo={"imdbid": "tt1234567"},
        )
        result = IdentificationResult(item=item, reasons=[])

        scorer = Scorer()
        score = scorer.score(result)

        assert "IMDB ID" in " ".join(score.reasons)
        assert score.confidence >= 50

    def test_nfo_tmdb_bonus(self):
        """Test TMDB ID bonus."""
        item = MediaItem(
            kind="movie",
            source_path="/path/file.mkv",
            title="Movie",
            nfo={"tmdbid": "603"},
        )
        result = IdentificationResult(item=item, reasons=[])

        scorer = Scorer()
        score = scorer.score(result)

        assert "TMDB ID" in " ".join(score.reasons)
        assert score.confidence >= 45

    def test_no_title_penalty(self):
        """Test penalty for missing title."""
        item = MediaItem(
            kind="movie",
            source_path="/path/file.mkv",
            title=None,
        )
        result = IdentificationResult(item=item, reasons=[])

        scorer = Scorer()
        score = scorer.score(result)

        assert "No title" in " ".join(score.reasons)

    def test_year_bonus(self):
        """Test year bonus for movies."""
        item1 = MediaItem(
            kind="movie",
            source_path="/path/file.mkv",
            title="Movie",
            year=2020,
        )
        item2 = MediaItem(
            kind="movie",
            source_path="/path/file.mkv",
            title="Movie",
            year=None,
        )

        result1 = IdentificationResult(item=item1, reasons=[])
        result2 = IdentificationResult(item=item2, reasons=[])

        scorer = Scorer()
        score1 = scorer.score(result1)
        score2 = scorer.score(result2)

        # Movie with year should have higher confidence
        assert score1.confidence > score2.confidence

    def test_error_penalty(self):
        """Test penalty for identification errors."""
        item = MediaItem(
            kind="movie",
            source_path="/path/file.mkv",
            title="Movie",
        )
        result = IdentificationResult(
            item=item,
            reasons=[],
            errors=["Failed to parse NFO", "Another error"],
        )

        scorer = Scorer()
        score = scorer.score(result)

        assert "error" in " ".join(score.reasons).lower()


class TestScorerIntegration:
    """Test scorer with real identification results."""

    def test_score_real_movie(self):
        """Test scoring a real movie identification."""
        result = identify_file("/movies/The Matrix (1999).mkv")

        scorer = Scorer()
        score = scorer.score(result)

        # Should have reasonable confidence
        assert 0 <= score.confidence <= 100
        assert len(score.reasons) > 0

    def test_score_real_episode(self):
        """Test scoring a real episode identification."""
        result = identify_file("/tv/Show/Season 01/Show.S01E05.mkv")

        scorer = Scorer()
        score = scorer.score(result)

        assert 0 <= score.confidence <= 100
        assert len(score.reasons) > 0


def test_score_identification_convenience():
    """Test convenience function."""
    item = MediaItem(
        kind="movie",
        source_path="/path/file.mkv",
        title="Movie",
        year=2020,
    )
    result = IdentificationResult(item=item, reasons=[])

    score = score_identification(result)

    assert score.confidence >= 0
    assert len(score.reasons) > 0
