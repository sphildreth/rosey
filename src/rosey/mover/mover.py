"""Transactional file mover with rollback support."""

import logging
import os
import shutil
from pathlib import Path
from typing import Literal

from rosey.models import MediaItem, MoveResult

logger = logging.getLogger(__name__)

# Use efficient syscall for cross-volume copies if available
try:
    _HAS_COPY_FILE_RANGE = hasattr(os, "copy_file_range")
except Exception:
    _HAS_COPY_FILE_RANGE = False

# All supported subtitle formats
SUBTITLE_EXTENSIONS = {
    ".srt",  # SubRip Text
    ".ssa",  # SubStation Alpha
    ".ass",  # Advanced SubStation Alpha
    ".vtt",  # WebVTT
    ".sub",  # VobSub / SubViewer / MicroDVD
    ".idx",  # VobSub index
    ".sbv",  # SubViewer
    ".lrc",  # LRC (Lyric file)
    ".smi",  # SAMI format
    ".stl",  # Spruce subtitle file
}

# All sidecar file types (subtitles + metadata + images)
SIDECAR_EXTENSIONS = SUBTITLE_EXTENSIONS | {".nfo", ".jpg", ".png", ".jpeg"}

# Directory creation cache to avoid redundant mkdir calls
_dir_created_cache: set[str] = set()


def _ensure_dir_exists(path: str) -> None:
    """Ensure directory exists, using cache to avoid redundant calls."""
    parent = str(Path(path).parent)
    if parent not in _dir_created_cache:
        Path(parent).mkdir(parents=True, exist_ok=True)
        _dir_created_cache.add(parent)


def _fast_copy_file_range(src: str, dst: str) -> bool:
    """Use copy_file_range for efficient zero-copy transfer."""
    if not _HAS_COPY_FILE_RANGE:
        return False

    src_fd = None
    dst_fd = None
    try:
        src_fd = os.open(src, os.O_RDONLY)
        dst_fd = os.open(dst, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
        chunk_size = 64 * 1024 * 1024
        while True:
            copied = os.copy_file_range(src_fd, dst_fd, chunk_size)
            if copied == 0:
                return True
    except Exception:
        return False
    finally:
        if dst_fd is not None:
            os.close(dst_fd)
        if src_fd is not None:
            os.close(src_fd)


def _fast_verify_files_identical(src: str, dst: str) -> bool:
    """Fast verification using copy_file_range to compare files."""
    if not _HAS_COPY_FILE_RANGE:
        # Fallback to size check
        return os.path.getsize(src) == os.path.getsize(dst)

    try:
        src_stat = os.stat(src)
        dst_stat = os.stat(dst)

        # Quick size check first
        if src_stat.st_size != dst_stat.st_size:
            return False

        # For small files, just use size check
        if src_stat.st_size < 1024 * 1024:  # < 1MB
            return True

        # Use copy_file_range to verify by reading through both files
        src_fd = os.open(src, os.O_RDONLY)
        try:
            dst_fd = os.open(dst, os.O_RDONLY)
            try:
                chunk_size = 64 * 1024 * 1024
                remaining = src_stat.st_size
                while remaining > 0:
                    to_copy = min(chunk_size, remaining)
                    # Verify by reading from both
                    src_hash = os.read(src_fd, to_copy)
                    dst_hash = os.read(dst_fd, to_copy)
                    if src_hash != dst_hash:
                        return False
                    remaining -= to_copy
            finally:
                os.close(dst_fd)
        finally:
            os.close(src_fd)
        return True
    except Exception:
        return False


def discover_sidecars(source_path: str) -> list[str]:
    """Find sidecar files that share the same base filename."""
    path = Path(source_path)
    base = path.stem
    parent = path.parent

    if not parent.exists():
        return []

    sidecars = []
    # Pre-compute lowercase suffix set for faster lookup
    sidecar_exts_lower = {ext.lower() for ext in SIDECAR_EXTENSIONS}

    for file in parent.iterdir():
        # Early filter: check suffix first (most selective)
        suffix_lower = file.suffix.lower()
        if (
            suffix_lower in sidecar_exts_lower
            and file.is_file()
            and file != path
            and file.stem == base
        ):
            sidecars.append(str(file))

    return sidecars


def check_preflight(
    source_paths: list[str], destination_dir: str
) -> dict[str, bool | str | list[str]]:
    """Perform preflight checks before moving files."""
    errors: list[str] = []

    dest_path = Path(destination_dir)

    # Check destination exists and is writable
    if not dest_path.exists():
        try:
            dest_path.mkdir(parents=True, exist_ok=True)
        except Exception as e:
            return {
                "free_space_ok": False,
                "path_len_ok": False,
                "perms_ok": False,
                "errors": [f"Cannot create destination: {e}"],
            }

    if not os.access(dest_path, os.W_OK):
        errors.append("Destination is not writable")

    # Calculate total size needed (batch stat calls)
    total_size = 0
    for src in source_paths:
        try:
            total_size += os.stat(src).st_size
        except Exception as e:
            logger.warning(f"Cannot stat {src}: {e}")

    # Check free space (with 100MB buffer)
    try:
        stat = shutil.disk_usage(dest_path)
        buffer = 100 * 1024 * 1024  # 100MB
        if stat.free < total_size + buffer:
            errors.append(f"Insufficient space: need {total_size + buffer} bytes, have {stat.free}")
    except Exception as e:
        logger.warning(f"Cannot check disk space: {e}")

    # Check path length (Windows has 260 char limit, but we'll use 255 as safe limit)
    dest_name_len = len(str(dest_path)) + 1  # +1 for separator
    for src in source_paths:
        if dest_name_len + len(Path(src).name) > 255:
            errors.append(f"Path too long: {src}")
            break

    return {
        "free_space_ok": "Insufficient space" not in str(errors),
        "path_len_ok": "Path too long" not in str(errors),
        "perms_ok": "Destination is not writable" not in str(errors),
        "errors": errors,
    }


def same_volume(source: str, dest: str) -> bool:
    """Check if source and destination are on the same volume."""
    try:
        src_stat = os.stat(source)
        dest_parent = str(Path(dest).parent)
        # Ensure destination directory exists using cache
        _ensure_dir_exists(dest)
        dest_stat = os.stat(dest_parent)
        return src_stat.st_dev == dest_stat.st_dev
    except Exception:
        return False


def apply_conflict_suffix(dest_path: str) -> str:
    """Apply (1), (2), etc. suffix for Keep Both policy."""
    path = Path(dest_path)
    parent = path.parent
    stem = path.stem
    suffix = path.suffix
    counter = 1

    while True:
        new_name = f"{stem} ({counter}){suffix}"
        new_path = parent / new_name
        if not new_path.exists():
            return str(new_path)
        counter += 1


def move_file_transactional(
    source: str,
    dest: str,
    conflict_policy: Literal["skip", "replace", "keep_both"] = "skip",
    dry_run: bool = True,
) -> tuple[bool, str]:
    """
    Move a single file with transactional guarantees.

    Returns (success, action_taken) where action_taken is one of:
    "moved", "skipped", "replaced", "kept_both"
    """
    if dry_run:
        logger.info(f"[DRY-RUN] Would move {source} -> {dest}")
        return True, "moved"

    dest_path = Path(dest)
    source_path = Path(source)

    if not source_path.exists():
        logger.error(f"Source does not exist: {source}")
        return False, "skipped"

    # Handle conflicts
    if dest_path.exists():
        if conflict_policy == "skip":
            logger.info(f"Skipping (exists): {dest}")
            return True, "skipped"
        elif conflict_policy == "keep_both":
            dest = apply_conflict_suffix(dest)
            dest_path = Path(dest)
            logger.info(f"Keep both: renamed to {dest}")
            action = "kept_both"
        else:  # replace
            logger.info(f"Replacing: {dest}")
            action = "replaced"
    else:
        action = "moved"

    # Ensure destination directory exists (uses cache internally)
    _ensure_dir_exists(dest)

    try:
        if same_volume(source, dest):
            # Atomic rename on same volume
            os.replace(source, dest)
            logger.info(f"Renamed {source} -> {dest}")
        else:
            # Cross-volume: copy, verify, then remove source
            # Try efficient copy first, fallback to shutil.copy2
            if not _fast_copy_file_range(source, dest):
                shutil.copy2(source, dest)

            # Verify copy integrity
            if not _fast_verify_files_identical(source, dest):
                dest_path.unlink(missing_ok=True)
                logger.error(f"Verification failed: {source} -> {dest}")
                return False, "skipped"

            # Verification passed - remove source
            source_path.unlink()
            logger.info(f"Copied and verified {source} -> {dest}")

        return True, action
    except Exception as e:
        logger.error(f"Failed to move {source} -> {dest}: {e}")
        # Clean up partial copy if exists
        if dest_path.exists() and not Path(source).exists():
            dest_path.unlink(missing_ok=True)
        return False, "skipped"


def move_with_sidecars(
    item: MediaItem,
    destination: str,
    conflict_policy: Literal["skip", "replace", "keep_both"] = "skip",
    dry_run: bool = True,
) -> MoveResult:
    """
    Move a media item and its sidecars transactionally.

    If any file fails to move, rollback by deleting any already-moved files.
    """
    source = item.source_path
    sidecars = discover_sidecars(source)

    all_sources = [source] + sidecars
    moved_files: list[str] = []
    details: dict[str, list[str]] = {"moved": [], "skipped": [], "replaced": [], "kept_both": []}
    errors: list[str] = []

    # Preflight check
    dest_dir = str(Path(destination).parent)
    preflight = check_preflight(all_sources, dest_dir)
    preflight_errors = preflight.get("errors", [])
    if not all([preflight["free_space_ok"], preflight["path_len_ok"], preflight["perms_ok"]]):
        return MoveResult(
            success=False,
            details=details,
            errors=preflight_errors
            if isinstance(preflight_errors, list)
            else ["Preflight checks failed"],
        )

    try:
        # Move main file
        success, action = move_file_transactional(source, destination, conflict_policy, dry_run)
        if success:
            details[action].append(destination)
            moved_files.append(destination)
        else:
            errors.append(f"Failed to move {source}")
            return MoveResult(success=False, details=details, errors=errors)

        # Move sidecars
        dest_parent = Path(destination).parent
        dest_stem = Path(destination).stem
        for sidecar in sidecars:
            sidecar_ext = Path(sidecar).suffix
            sidecar_dest = dest_parent / f"{dest_stem}{sidecar_ext}"

            success, action = move_file_transactional(
                sidecar, str(sidecar_dest), conflict_policy, dry_run
            )
            if success:
                details[action].append(str(sidecar_dest))
                moved_files.append(str(sidecar_dest))
            else:
                # Rollback: delete moved files
                if not dry_run:
                    for moved in moved_files:
                        try:
                            Path(moved).unlink(missing_ok=True)
                            logger.info(f"Rolled back: {moved}")
                        except Exception as e:
                            logger.error(f"Rollback failed for {moved}: {e}")

                errors.append(f"Failed to move sidecar {sidecar}, rolled back")
                return MoveResult(
                    success=False,
                    details=details,
                    errors=errors,
                    rollback_performed=True,
                )

        return MoveResult(success=True, details=details)

    except Exception as e:
        # Rollback on unexpected error
        if not dry_run:
            for moved in moved_files:
                try:
                    Path(moved).unlink(missing_ok=True)
                    logger.info(f"Rolled back: {moved}")
                except Exception as rollback_err:
                    logger.error(f"Rollback failed for {moved}: {rollback_err}")

        errors.append(f"Unexpected error: {e}")
        return MoveResult(
            success=False,
            details=details,
            errors=errors,
            rollback_performed=True,
        )
