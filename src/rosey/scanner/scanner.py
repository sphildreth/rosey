"""File system scanner with concurrency and error handling."""

import logging
from collections.abc import Iterator
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

logger = logging.getLogger(__name__)

# Video file extensions
VIDEO_EXTENSIONS = {
    ".mkv",
    ".mp4",
    ".avi",
    ".mov",
    ".wmv",
    ".flv",
    ".m4v",
    ".mpg",
    ".mpeg",
    ".webm",
    ".ts",
}


class ScanResult:
    """Result from scanning a path."""

    def __init__(self, path: str, is_video: bool, size_bytes: int = 0, error: str | None = None):
        self.path = path
        self.is_video = is_video
        self.size_bytes = size_bytes
        self.error = error


class Scanner:
    """Scans filesystem for media files."""

    def __init__(self, max_workers: int = 8, follow_symlinks: bool = False):
        """
        Initialize scanner.

        Args:
            max_workers: Maximum number of concurrent workers
            follow_symlinks: Whether to follow symbolic links
        """
        self.max_workers = max_workers
        self.follow_symlinks = follow_symlinks

    def scan(self, root_path: str) -> list[ScanResult]:
        """
        Scan a directory tree for media files.

        Args:
            root_path: Root directory to scan

        Returns:
            List of ScanResult objects
        """
        results: list[ScanResult] = []
        paths_to_scan = list(self._enumerate_paths(root_path))

        if not paths_to_scan:
            logger.warning(f"No files found in {root_path}")
            return results

        # Process paths concurrently
        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            future_to_path = {
                executor.submit(self._scan_path, path): path for path in paths_to_scan
            }

            for future in as_completed(future_to_path):
                try:
                    result = future.result()
                    if result:
                        results.append(result)
                except Exception as e:
                    path = future_to_path[future]
                    logger.error(f"Error scanning {path}: {e}")
                    results.append(ScanResult(path, False, error=str(e)))

        logger.info(f"Scanned {len(results)} files from {root_path}")
        return results

    def _enumerate_paths(self, root_path: str) -> Iterator[str]:
        """
        Enumerate all file paths under root_path.

        Args:
            root_path: Root directory to enumerate

        Yields:
            File paths
        """
        root = Path(root_path)

        if not root.exists():
            logger.error(f"Path does not exist: {root_path}")
            return

        if not root.is_dir():
            # Single file
            yield str(root)
            return

        try:
            for entry in root.rglob("*"):
                if not self.follow_symlinks and entry.is_symlink():
                    continue

                if entry.is_file():
                    yield str(entry)
        except PermissionError as e:
            logger.error(f"Permission denied accessing {root_path}: {e}")
        except Exception as e:
            logger.error(f"Error enumerating {root_path}: {e}")

    def _scan_path(self, path: str) -> ScanResult | None:
        """
        Scan a single file path.

        Args:
            path: File path to scan

        Returns:
            ScanResult or None if path should be skipped
        """
        try:
            p = Path(path)
            ext = p.suffix.lower()
            is_video = ext in VIDEO_EXTENSIONS

            # Get file size
            size = 0
            if is_video:
                try:
                    size = p.stat().st_size
                except Exception as e:
                    logger.warning(f"Could not get size for {path}: {e}")

            return ScanResult(path, is_video, size)

        except PermissionError as e:
            logger.error(f"Permission denied: {path}")
            return ScanResult(path, False, error=f"Permission denied: {e}")
        except Exception as e:
            logger.error(f"Error scanning {path}: {e}")
            return ScanResult(path, False, error=str(e))


def scan_directory(
    root_path: str,
    max_workers: int = 8,
    follow_symlinks: bool = False,
) -> list[ScanResult]:
    """
    Convenience function to scan a directory.

    Args:
        root_path: Root directory to scan
        max_workers: Maximum concurrent workers
        follow_symlinks: Whether to follow symbolic links

    Returns:
        List of ScanResult objects
    """
    scanner = Scanner(max_workers=max_workers, follow_symlinks=follow_symlinks)
    return scanner.scan(root_path)
