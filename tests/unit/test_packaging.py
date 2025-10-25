"""Tests for packaging configuration and smoke tests."""

import os
from pathlib import Path

import pytest


class TestPackaging:
    """Test packaging configuration and build scripts."""

    def test_pyinstaller_spec_exists(self) -> None:
        """PyInstaller spec file should exist at repo root."""
        repo_root = Path(__file__).parent.parent.parent
        spec_file = repo_root / "rosey.spec"
        assert spec_file.exists(), "rosey.spec not found"
        assert spec_file.is_file(), "rosey.spec is not a file"

    def test_spec_file_valid_syntax(self) -> None:
        """PyInstaller spec file should have valid Python syntax."""
        repo_root = Path(__file__).parent.parent.parent
        spec_file = repo_root / "rosey.spec"

        with open(spec_file) as f:
            spec_content = f.read()

        # Should not raise SyntaxError
        compile(spec_content, str(spec_file), "exec")

    def test_spec_includes_entry_point(self) -> None:
        """Spec should reference the app entry point."""
        repo_root = Path(__file__).parent.parent.parent
        spec_file = repo_root / "rosey.spec"

        with open(spec_file) as f:
            content = f.read()

        assert "src/rosey/app.py" in content, "Entry point not specified in spec"

    def test_spec_includes_graphics(self) -> None:
        """Spec should bundle graphics/icons."""
        repo_root = Path(__file__).parent.parent.parent
        spec_file = repo_root / "rosey.spec"

        with open(spec_file) as f:
            content = f.read()

        assert "graphics" in content, "Graphics not included in spec"
        assert "rosey.ico" in content, "Windows icon not included"
        assert "rosey_256.png" in content, "Linux icon not included"

    def test_spec_includes_hiddenimports(self) -> None:
        """Spec should declare hidden imports for all rosey modules."""
        repo_root = Path(__file__).parent.parent.parent
        spec_file = repo_root / "rosey.spec"

        with open(spec_file) as f:
            content = f.read()

        # Check for key modules
        required_modules = [
            "rosey.scanner",
            "rosey.identifier",
            "rosey.scorer",
            "rosey.planner",
            "rosey.mover",
            "rosey.providers",
            "rosey.ui",
        ]

        for module in required_modules:
            assert module in content, f"Module {module} not in hiddenimports"

    def test_graphics_files_exist(self) -> None:
        """Required icon/graphic files should exist."""
        repo_root = Path(__file__).parent.parent.parent
        graphics_dir = repo_root / "graphics"

        assert graphics_dir.exists(), "graphics/ directory not found"

        required_files = [
            "rosey.ico",  # Windows icon
            "rosey_256.png",  # Linux icon
            "rosey_64.png",  # Smaller icon
        ]

        for filename in required_files:
            filepath = graphics_dir / filename
            assert filepath.exists(), f"Graphics file {filename} not found"
            assert filepath.stat().st_size > 0, f"Graphics file {filename} is empty"

    def test_build_scripts_exist(self) -> None:
        """Build scripts should exist for Windows and Linux."""
        repo_root = Path(__file__).parent.parent.parent
        scripts_dir = repo_root / "scripts"

        build_linux = scripts_dir / "build_package.sh"
        build_windows = scripts_dir / "build_package.bat"

        assert build_linux.exists(), "build_package.sh not found"
        assert build_windows.exists(), "build_package.bat not found"

    def test_smoke_test_scripts_exist(self) -> None:
        """Smoke test scripts should exist for Windows and Linux."""
        repo_root = Path(__file__).parent.parent.parent
        scripts_dir = repo_root / "scripts"

        smoke_linux = scripts_dir / "smoke_test.sh"
        smoke_windows = scripts_dir / "smoke_test.bat"

        assert smoke_linux.exists(), "smoke_test.sh not found"
        assert smoke_windows.exists(), "smoke_test.bat not found"

    @pytest.mark.skipif(os.name != "posix", reason="Unix-only test")
    def test_build_script_executable_unix(self) -> None:
        """Build script should be executable on Unix."""
        repo_root = Path(__file__).parent.parent.parent
        build_script = repo_root / "scripts" / "build_package.sh"

        assert os.access(build_script, os.X_OK), "build_package.sh not executable"

    @pytest.mark.skipif(os.name != "posix", reason="Unix-only test")
    def test_smoke_test_script_executable_unix(self) -> None:
        """Smoke test script should be executable on Unix."""
        repo_root = Path(__file__).parent.parent.parent
        smoke_script = repo_root / "scripts" / "smoke_test.sh"

        assert os.access(smoke_script, os.X_OK), "smoke_test.sh not executable"

    def test_smoke_test_checks_binary_existence(self) -> None:
        """Smoke test should check if binary exists."""
        repo_root = Path(__file__).parent.parent.parent
        smoke_linux = repo_root / "scripts" / "smoke_test.sh"

        with open(smoke_linux) as f:
            content = f.read()

        assert "dist/rosey/rosey" in content, "Smoke test doesn't check binary path"
        assert "not found" in content.lower() or "not exist" in content.lower()

    def test_app_entry_point_exists(self) -> None:
        """App entry point module should exist and be importable."""
        repo_root = Path(__file__).parent.parent.parent
        app_file = repo_root / "src" / "rosey" / "app.py"

        assert app_file.exists(), "src/rosey/app.py not found"

        # Check it has main() function
        with open(app_file) as f:
            content = f.read()

        assert "def main()" in content, "main() function not found in app.py"
        assert "QApplication" in content, "QApplication not used in app.py"

    def test_version_defined(self) -> None:
        """Package version should be defined."""
        repo_root = Path(__file__).parent.parent.parent
        init_file = repo_root / "src" / "rosey" / "__init__.py"

        with open(init_file) as f:
            content = f.read()

        assert "__version__" in content, "__version__ not defined"

    def test_setup_documentation_exists(self) -> None:
        """User setup documentation should exist."""
        repo_root = Path(__file__).parent.parent.parent
        docs_dir = repo_root / "docs"

        setup_doc = docs_dir / "SETUP.md"
        assert setup_doc.exists(), "docs/SETUP.md not found"

        with open(setup_doc) as f:
            content = f.read()

        # Check for key sections
        assert "Installation" in content, "Installation section missing"
        assert "Windows" in content, "Windows instructions missing"
        assert "Linux" in content, "Linux instructions missing"
        assert "Troubleshooting" in content, "Troubleshooting section missing"

    def test_setup_doc_covers_requirements(self) -> None:
        """Setup doc should cover system requirements."""
        repo_root = Path(__file__).parent.parent.parent
        setup_doc = repo_root / "docs" / "SETUP.md"

        with open(setup_doc) as f:
            content = f.read()

        assert "System Requirements" in content or "Requirements" in content
        assert "Windows" in content and "Linux" in content

    def test_setup_doc_covers_paths(self) -> None:
        """Setup doc should explain path configuration."""
        repo_root = Path(__file__).parent.parent.parent
        setup_doc = repo_root / "docs" / "SETUP.md"

        with open(setup_doc) as f:
            content = f.read()

        assert "path" in content.lower()
        assert "source" in content.lower()
        assert "destination" in content.lower()

    def test_setup_doc_covers_troubleshooting_paths(self) -> None:
        """Setup doc should have troubleshooting for path issues."""
        repo_root = Path(__file__).parent.parent.parent
        setup_doc = repo_root / "docs" / "SETUP.md"

        with open(setup_doc) as f:
            content = f.read()

        # Should mention common issues
        assert "permission" in content.lower()
        assert "network share" in content.lower() or "network" in content.lower()

    def test_setup_doc_covers_troubleshooting_network(self) -> None:
        """Setup doc should have troubleshooting for network shares."""
        repo_root = Path(__file__).parent.parent.parent
        setup_doc = repo_root / "docs" / "SETUP.md"

        with open(setup_doc) as f:
            content = f.read()

        content_lower = content.lower()
        assert "network" in content_lower
        # Should mention UNC paths or mounting
        assert "unc" in content_lower or "mount" in content_lower or "smb" in content_lower


class TestSmokeTestLogic:
    """Test the smoke test script logic."""

    def test_smoke_test_has_platform_detection(self) -> None:
        """Smoke test should detect platform-specific binary names."""
        repo_root = Path(__file__).parent.parent.parent

        # Check Linux script
        smoke_linux = repo_root / "scripts" / "smoke_test.sh"
        with open(smoke_linux) as f:
            linux_content = f.read()

        # Should reference Linux binary (no .exe)
        assert "rosey.exe" not in linux_content

        # Check Windows script
        smoke_windows = repo_root / "scripts" / "smoke_test.bat"
        with open(smoke_windows) as f:
            windows_content = f.read()

        # Should reference Windows binary (.exe)
        assert "rosey.exe" in windows_content

    def test_smoke_test_has_manual_checklist(self) -> None:
        """Smoke test should provide manual test checklist."""
        repo_root = Path(__file__).parent.parent.parent
        smoke_linux = repo_root / "scripts" / "smoke_test.sh"

        with open(smoke_linux) as f:
            content = f.read()

        checklist_keywords = ["scan", "dry-run", "exit"]
        for keyword in checklist_keywords:
            assert keyword.lower() in content.lower(), f"Manual checklist missing '{keyword}'"


class TestBuildScriptLogic:
    """Test the build script logic."""

    def test_build_script_references_spec(self) -> None:
        """Build script should reference rosey.spec."""
        repo_root = Path(__file__).parent.parent.parent
        build_linux = repo_root / "scripts" / "build_package.sh"

        with open(build_linux) as f:
            content = f.read()

        assert "rosey.spec" in content, "Build script doesn't reference rosey.spec"
        assert "pyinstaller" in content.lower()

    def test_build_script_installs_pyinstaller(self) -> None:
        """Build script should install PyInstaller."""
        repo_root = Path(__file__).parent.parent.parent
        build_linux = repo_root / "scripts" / "build_package.sh"

        with open(build_linux) as f:
            content = f.read()

        assert "pip install pyinstaller" in content.lower()
