import shutil
from pathlib import Path

from rosey.identifier.identifier import Identifier


def setup_test_env(base_path: Path, structure: dict):
    if base_path.exists():
        shutil.rmtree(base_path)
    base_path.mkdir(parents=True)

    for name, content in structure.items():
        if isinstance(content, dict):
            setup_test_env(base_path / name, content)
        else:
            (base_path / name).touch()


def test_scenario(name: str, structure: dict, movie_filename: str):
    print(f"\nTesting scenario: {name}")
    base_path = Path(f"temp_test_{name}")
    try:
        setup_test_env(base_path, structure)

        movie_path = base_path / movie_filename
        identifier = Identifier()
        result = identifier.identify(str(movie_path))

        print(f"Movie: {result.item.title}")
        print(f"Sidecars found: {len(result.item.sidecars)}")
        for sidecar in result.item.sidecars:
            print(f" - {Path(sidecar).relative_to(base_path)}")

        return result.item.sidecars
    finally:
        if base_path.exists():
            shutil.rmtree(base_path)


# Scenario 1: Standard "Subs" folder
structure_1 = {"Movie (2023).mkv": None, "Subs": {"en.srt": None}}

# Scenario 2: Lowercase "subs" folder
structure_2 = {"Movie (2023).mkv": None, "subs": {"en.srt": None}}

# Scenario 3: "Subtitles" folder
structure_3 = {"Movie (2023).mkv": None, "Subtitles": {"en.srt": None}}

# Scenario 4: Nested in "Subs"
structure_4 = {"Movie (2023).mkv": None, "Subs": {"English": {"en.srt": None}}}

if __name__ == "__main__":
    test_scenario("Standard_Subs", structure_1, "Movie (2023).mkv")
    test_scenario("Lowercase_subs", structure_2, "Movie (2023).mkv")
    test_scenario("Subtitles_folder", structure_3, "Movie (2023).mkv")
    test_scenario("Nested_in_Subs", structure_4, "Movie (2023).mkv")
