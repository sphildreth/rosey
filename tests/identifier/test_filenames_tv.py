from pathlib import Path
from typing import Any

from tests import conftest as helpers


def pytest_generate_tests(metafunc):
    if "row" in metafunc.fixturenames:
        fixtures_dir = Path(__file__).parent / "fixtures"
        rows = helpers.load_csv_fixture(fixtures_dir / "filenames_tv.csv")
        metafunc.parametrize("row", rows, ids=[r["source_path"] for r in rows])


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


def test_tv_filename_parsing(row):
    parser = helpers.import_or_skip_parser()

    path = row["source_path"]
    expected_kind = row["kind"].strip() or None
    expected_title = row["title"].strip() or None
    expected_year = helpers.parse_int(row.get("year"))
    expected_season = helpers.parse_int(row.get("season"))
    expected_eps = helpers.parse_list_of_ints(row.get("episodes"))
    expected_part = helpers.parse_int(row.get("part"))
    expected_date = (row.get("date") or "").strip() or None
    reason_needle = (row.get("reason_contains") or "").strip() or None

    parsed = parser(path)
    item = helpers.normalize_item(parsed)

    assert item["kind"] == expected_kind
    assert item["title"] == expected_title
    assert item["year"] == expected_year
    assert item["season"] == expected_season
    assert item["episodes"] == expected_eps
    assert item["part"] == expected_part
    assert item["date"] == expected_date

    reasons = _get_reasons(parsed)
    helpers.assert_reason_contains(reasons, reason_needle)
