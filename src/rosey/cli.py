"""Command-line interface for Rosey."""

import argparse
import logging
import sys
from pathlib import Path

from rosey.config import load_config, save_config
from rosey.identifier import identify_file
from rosey.mover import move_with_sidecars
from rosey.planner import plan_path
from rosey.scanner import scan_directory
from rosey.scorer import score_identification

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


def main() -> int:
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Rosey - Media file organizer",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        "source",
        nargs="?",
        help="Source directory to scan",
    )

    parser.add_argument(
        "--movies-target",
        help="Target directory for movies",
    )

    parser.add_argument(
        "--tv-target",
        help="Target directory for TV shows",
    )

    parser.add_argument(
        "--dry-run",
        action="store_true",
        default=True,
        help="Dry-run mode (default: True)",
    )

    parser.add_argument(
        "--no-dry-run",
        dest="dry_run",
        action="store_false",
        help="Disable dry-run mode",
    )

    parser.add_argument(
        "--max-workers",
        type=int,
        default=8,
        help="Maximum concurrent workers for scanning (default: 8)",
    )

    parser.add_argument(
        "--confidence",
        type=int,
        default=0,
        help="Minimum confidence threshold to display (0-100, default: 0)",
    )

    parser.add_argument(
        "--save-config",
        action="store_true",
        help="Save provided paths to config file",
    )

    parser.add_argument(
        "--conflict-policy",
        choices=["skip", "replace", "keep_both"],
        default="skip",
        help="Conflict policy for live moves (default: skip)",
    )

    args = parser.parse_args()

    # Load config
    config = load_config()

    # Determine paths (CLI args override config)
    source = args.source or config.paths.source
    movies_target = args.movies_target or config.paths.movies
    tv_target = args.tv_target or config.paths.tv

    if not source:
        parser.print_help()
        print("\nError: No source directory specified. Use --source or set in config.")
        return 1

    # Save config if requested
    if args.save_config:
        if args.source:
            config.paths.source = args.source
        if args.movies_target:
            config.paths.movies = args.movies_target
        if args.tv_target:
            config.paths.tv = args.tv_target
        save_config(config)
        logger.info("Configuration saved")

    # Validate source exists
    if not Path(source).exists():
        logger.error(f"Source directory does not exist: {source}")
        return 1

    logger.info(f"Scanning: {source}")
    logger.info(f"Movies target: {movies_target or '(not set)'}")
    logger.info(f"TV target: {tv_target or '(not set)'}")
    logger.info(f"Dry-run: {args.dry_run}")
    logger.info("=" * 80)

    # Scan
    logger.info("Step 1: Scanning filesystem...")
    scan_results = scan_directory(source, max_workers=args.max_workers)

    # Filter for video files
    video_files = [r for r in scan_results if r.is_video and not r.error]

    logger.info(f"Found {len(video_files)} video files")

    if not video_files:
        logger.warning("No video files found")
        return 0

    # Identify and score
    logger.info("\nStep 2: Identifying media files...")
    results: list[dict] = []

    for scan_result in video_files:
        # Identify
        ident_result = identify_file(scan_result.path)

        # Score
        score_result = score_identification(ident_result)

        # Skip if below confidence threshold
        if score_result.confidence < args.confidence:
            continue

        # Plan destination
        destination = plan_path(
            ident_result.item,
            movies_root=movies_target,
            tv_root=tv_target,
        )

        results.append(
            {
                "item": ident_result.item,
                "score": score_result,
                "destination": destination,
            }
        )

    # Display results
    logger.info(f"\nResults: {len(results)} items")
    logger.info("=" * 80)

    # Group by confidence
    green = [r for r in results if r["score"].confidence >= 70]
    yellow = [r for r in results if 40 <= r["score"].confidence < 70]
    red = [r for r in results if r["score"].confidence < 40]

    logger.info(f"Confidence: Green={len(green)}, Yellow={len(yellow)}, Red={len(red)}")
    logger.info("")

    for result in results:
        item = result["item"]
        score_result = result["score"]
        dest = result["destination"]

        # Confidence label
        if score_result.confidence >= 70:
            label = "GREEN"
        elif score_result.confidence >= 40:
            label = "YELLOW"
        else:
            label = "RED"

        # Basic info
        if item.kind == "movie":
            info = f"{item.title} ({item.year or '?'})"
        elif item.kind == "episode":
            if item.episodes:
                ep_str = f"S{item.season:02d}E{item.episodes[0]:02d}"
                info = f"{item.title} - {ep_str}"
            elif item.date:
                info = f"{item.title} - {item.date}"
            else:
                info = f"{item.title}"
        else:
            info = "Unknown"

        logger.info(f"[{label}] {score_result.confidence:3d}% | {info}")
        logger.info(f"  Source: {item.source_path}")
        logger.info(f"  Dest:   {dest}")
        logger.info(f"  Reasons: {'; '.join(score_result.reasons)}")
        logger.info("")

    if args.dry_run:
        logger.info("DRY-RUN mode - no files were moved")
    else:
        logger.info("\nStep 3: Moving files (LIVE mode)")
        moved = 0
        errors: list[str] = []
        for result in results:
            item = result["item"]
            dest = result["destination"]
            try:
                move_result = move_with_sidecars(
                    item,
                    dest,
                    conflict_policy=args.conflict_policy,
                    dry_run=False,
                )
                if move_result.success:
                    moved += 1
                    logger.info(f"Moved: {dest}")
                else:
                    for err in move_result.errors:
                        logger.error(f"Move error: {err}")
                    errors.extend(move_result.errors)
            except Exception as e:  # pragma: no cover
                logger.error(f"Unexpected move error for {item.source_path}: {e}")
                errors.append(str(e))

        logger.info(f"Move summary: {moved}/{len(results)} succeeded; {len(errors)} errors")

    return 0


if __name__ == "__main__":
    sys.exit(main())
