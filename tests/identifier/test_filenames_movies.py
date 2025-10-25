from pathlib import Path
from typing import Any

from tests import conftest as helpers


def _get_reasons(parsed: Any) -> list[str]:
    if hasattr(parsed, "reasons"):
        r = parsed.reasons  # type: ignore[attr-defined]
        if isinstance(r, list):
            return [str(x) for x in r]
        return [str(r)]
    try:
        d = dict(parsed)
        r = d.get("reasons", [])
        if isinstance(r, list):
            return [str(x) for x in r]
    except Exception:
        pass
    return []


def test_movie_filename_parsing(fixtures_dir: Path):
    parser = helpers.import_or_skip_parser()
    rows = helpers.load_csv_fixture(fixtures_dir / "filenames_movies.csv")

    for row in rows:
        path = row["source_path"]
        expected_kind = row["kind"].strip() or None
        expected_title = row["title"].strip() or None
        expected_year = helpers.parse_int(row.get("year"))
        expected_part = helpers.parse_int(row.get("part"))

        parsed = parser(path)
        item = helpers.normalize_item(parsed)

        assert item["kind"] == expected_kind
        assert item["title"] == expected_title
        assert item["year"] == expected_year
        assert item["part"] == expected_part

        # Movies should not have season/episodes/date in these fixtures
        assert item["season"] is None
        assert item["episodes"] is None
        assert item["date"] is None

        reasons = _get_reasons(parsed)
        # We don't enforce specific reason phrases for stability
        assert isinstance(reasons, list)
