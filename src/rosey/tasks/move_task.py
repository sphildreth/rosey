"""Move task for background operations."""

import logging
from typing import Literal

from PySide6.QtCore import QObject, QRunnable, Signal

from rosey.models import MediaItem
from rosey.mover import move_with_sidecars

logger = logging.getLogger(__name__)


class MoveSignals(QObject):
    """Signals for move operations."""

    progress = Signal(int, int, str)  # current, total, message
    item_moved = Signal(str, bool, str)  # path, success, action
    finished = Signal(list)  # list of MoveResult
    error = Signal(str)


class MoveTask(QRunnable):
    """Background task for moving files."""

    def __init__(
        self,
        items: list[tuple[MediaItem, str]],  # (item, destination)
        conflict_policy: Literal["skip", "replace", "keep_both"] = "skip",
        dry_run: bool = True,
    ):
        super().__init__()
        self.items = items
        self.conflict_policy = conflict_policy
        self.dry_run = dry_run
        self.signals = MoveSignals()
        self._cancelled = False

    def cancel(self) -> None:
        """Request cancellation."""
        self._cancelled = True

    def run(self) -> None:
        """Execute the move operation."""
        results = []
        total = len(self.items)

        try:
            for idx, (item, destination) in enumerate(self.items):
                if self._cancelled:
                    self.signals.progress.emit(idx, total, "Cancelled")
                    break

                # Update progress
                self.signals.progress.emit(
                    idx,
                    total,
                    f"Moving {item.source_path}...",
                )

                # Perform move
                result = move_with_sidecars(
                    item,
                    destination,
                    conflict_policy=self.conflict_policy,
                    dry_run=self.dry_run,
                )

                results.append(result)

                # Emit item result
                action = "moved" if result.success else "failed"
                if result.details.get("skipped"):
                    action = "skipped"
                elif result.details.get("replaced"):
                    action = "replaced"
                elif result.details.get("kept_both"):
                    action = "kept_both"

                self.signals.item_moved.emit(item.source_path, result.success, action)

                if not result.success:
                    error_msg = "; ".join(result.errors)
                    self.signals.error.emit(f"Failed to move {item.source_path}: {error_msg}")

            self.signals.progress.emit(total, total, "Complete")

        except Exception as e:
            logger.exception("Move task failed")
            self.signals.error.emit(f"Move operation failed: {e}")

        finally:
            self.signals.finished.emit(results)
