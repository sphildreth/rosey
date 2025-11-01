"""Transactional file mover with rollback support."""

import logging
import os
import shutil
from pathlib import Path
from typing import Literal

from rosey.models import MediaItem, MoveResult

logger = logging.getLogger(__name__)

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


def discover_sidecars(source_path: str) -> list[str]:
    """Find sidecar files that share the same base filename."""
    path = Path(source_path)
    base = path.stem
    parent = path.parent
    sidecars: list[str] = []

    if not parent.exists():
        return sidecars

    for file in parent.iterdir():
        if (
            file.is_file()
            and file != path
            and file.stem == base
            and file.suffix.lower() in SIDECAR_EXTENSIONS
        ):
            sidecars.append(str(file))

    return sidecars


def check_preflight(
    source_paths: list[str], destination_dir: str
) -> dict[str, bool | str | list[str]]:
    """Perform preflight checks before moving files."""
    result: dict[str, bool | str | list[str]] = {
        "free_space_ok": True,
        "path_len_ok": True,
        "perms_ok": True,
        "errors": [],
    }

    dest_path = Path(destination_dir)

    # Check destination exists and is writable
    if not dest_path.exists():
        try:
            dest_path.mkdir(parents=True, exist_ok=True)
        except Exception as e:
            result["perms_ok"] = False
            errors = result.get("errors", [])
            if isinstance(errors, list):
                errors.append(f"Cannot create destination: {e}")
            return result

    if not os.access(dest_path, os.W_OK):
        result["perms_ok"] = False
        errors = result.get("errors", [])
        if isinstance(errors, list):
            errors.append("Destination is not writable")

    # Calculate total size needed
    total_size = 0
    for src in source_paths:
        try:
            total_size += Path(src).stat().st_size
        except Exception as e:
            logger.warning(f"Cannot stat {src}: {e}")

    # Check free space (with 100MB buffer)
    try:
        stat = shutil.disk_usage(dest_path)
        buffer = 100 * 1024 * 1024  # 100MB
        if stat.free < total_size + buffer:
            result["free_space_ok"] = False
            errors = result.get("errors", [])
            if isinstance(errors, list):
                errors.append(
                    f"Insufficient space: need {total_size + buffer} bytes, have {stat.free}"
                )
    except Exception as e:
        logger.warning(f"Cannot check disk space: {e}")

    # Check path length (Windows has 260 char limit, but we'll use 255 as safe limit)
    for src in source_paths:
        if len(str(dest_path / Path(src).name)) > 255:
            result["path_len_ok"] = False
            errors = result.get("errors", [])
            if isinstance(errors, list):
                errors.append(f"Path too long: {src}")
            break

    return result


def same_volume(source: str, dest: str) -> bool:
    """Check if source and destination are on the same volume."""
    try:
        src_stat = os.stat(source)
        dest_parent = Path(dest).parent
        if not dest_parent.exists():
            dest_parent.mkdir(parents=True, exist_ok=True)
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

    # Ensure destination directory exists
    dest_path.parent.mkdir(parents=True, exist_ok=True)

    try:
        if same_volume(source, dest):
            # Atomic rename on same volume
            os.replace(source, dest)
            logger.info(f"Renamed {source} -> {dest}")
        else:
            # Cross-volume: copy, verify, then remove source
            shutil.copy2(source, dest)

            # Verify size
            src_size = source_path.stat().st_size
            dest_size = dest_path.stat().st_size
            if src_size != dest_size:
                # Verification failed - remove partial copy
                dest_path.unlink(missing_ok=True)
                logger.error(f"Size mismatch: {source} ({src_size}) vs {dest} ({dest_size})")
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
