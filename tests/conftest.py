import csv
from collections.abc import Iterable
from pathlib import Path
from typing import Any

import pytest


def import_or_skip_parser():
    """Import the identifier parser, or skip tests if it's not implemented yet.

    Tries common function names on module rosey.identifier:
    - parse_path(path: str) -> dict|model
    - parse_media(path: str) -> dict|model
    - parse_filename(path: str) -> dict|model
    Returns a callable that accepts a path string and returns a dict-like object.
    """
    mod = pytest.importorskip("rosey.identifier", reason="identifier module not yet implemented")
    for fname in ("parse_path", "parse_media", "parse_filename"):
        fn = getattr(mod, fname, None)
        if callable(fn):
            return fn
    # Fallback to convenience identifier if available
    identify_file = getattr(mod, "identify_file", None)
    if callable(identify_file):
        # Adapt to expected callable signature returning an item-like object
        return lambda p: identify_file(p).item
    pytest.skip(
        "No parser function found in rosey.identifier (expected one of parse_path/parse_media/parse_filename or identify_file)"
    )


def import_or_skip_planner():
    """Import the path planner, or skip if not implemented."""
    mod = pytest.importorskip("rosey.planner", reason="planner module not yet implemented")
    for fname in ("plan_destination", "build_paths", "plan_paths"):
        fn = getattr(mod, fname, None)
        if callable(fn):
            return fn
    pytest.skip(
        "No planner function found in rosey.planner (expected one of plan_destination/build_paths/plan_paths)"
    )


def load_csv_fixture(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            rows.append(row)
    return rows


def parse_int(value: str | None) -> int | None:
    if value is None:
        return None
    value = value.strip()
    if not value:
        return None
    try:
        return int(value)
    except ValueError:
        return None


def parse_list_of_ints(value: str | None) -> list[int] | None:
    if value is None:
        return None
    txt = value.strip()
    if not txt:
        return None
    # Support comma or pipe
    parts = [p.strip() for p in txt.replace(",", "|").split("|") if p.strip()]
    out: list[int] = []
    for p in parts:
        try:
            out.append(int(p))
        except ValueError:
            # tolerate non-int in fixtures by skipping
            continue
    return out or None


def normalize_item(item: Any) -> dict[str, Any]:
    """Convert parser return (dict or model) to a plain dict with expected keys."""
    # If pydantic model or similar, try .model_dump()/dict()
    if hasattr(item, "model_dump"):
        data = item.model_dump()
    elif hasattr(item, "dict") and callable(item.dict):  # type: ignore[attr-defined]
        data = item.dict()  # type: ignore[call-arg]
    else:
        data = dict(item)

    # Expected keys
    expected_keys = {
        "kind",
        "title",
        "year",
        "season",
        "episodes",
        "part",
        "date",
    }
    # Filter extra keys to keep assertions stable
    return {k: data.get(k) for k in expected_keys}


def assert_reason_contains(reasons: Iterable[str], needle: str | None):
    if not needle:
        return
    hay = " ".join(reasons).lower()
    assert needle.lower() in hay, f"Expected reason to contain '{needle}', got: {hay}"


@pytest.fixture
def fixtures_dir() -> Path:
    return Path(__file__).parent / "identifier" / "fixtures"
