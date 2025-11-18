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


def test_scenario(name: str, structure: dict, target_file: str):
    print(f"\nTesting scenario: {name}")
    base_path = Path(f"temp_test_{name}")
    try:
        setup_test_env(base_path, structure)

        file_path = base_path / target_file
        identifier = Identifier()
        # We want to check if it identifies as an episode (TV Show) or Movie
        result = identifier.identify(str(file_path))

        print(f"Type: {result.item.type}")
        print(f"Title: {result.item.title}")
        if result.item.season:
            print(f"Season: {result.item.season}")
        if result.item.episode:
            print(f"Episode: {result.item.episode}")

        return result.item.type
    finally:
        if base_path.exists():
            shutil.rmtree(base_path)


# Scenario: "Version 5.0" season folder with single episode
structure_it_crowd = {
    "The IT Crowd (2006) [tmdbid-2490]": {
        "Season 1": {"S01E01.mkv": None},
        "Version 5.0": {"The IT Crowd - Version 5.0.mkv": None},
    }
}

if __name__ == "__main__":
    test_scenario(
        "IT_Crowd_Version_5",
        structure_it_crowd,
        "The IT Crowd (2006) [tmdbid-2490]/Version 5.0/The IT Crowd - Version 5.0.mkv",
    )
