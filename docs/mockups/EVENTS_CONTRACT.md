
# Event & Signal Contract (for agents)

## Commands (triggered by buttons / menu)
- `scanRequested(sourceDir)`
- `moveRequested(selectedItems)`  // execute plan for checked rows (alias of runRequested)
- `openConfigRequested()`
- `selectAllGreenRequested()`
- `clearRequested()`
- `openInFileManager(path)`
- `revealDestination(path)`

## Long-running tasks → signals
- `scanProgress(foundCount, currentPath)`
- `scanCompleted(treeModel, flatItems)`
- `lookupProgress(done, total)`
- `moveProgress(bytesMoved, fileIndex, totalFiles, currentSrc, currentDst)`
- `moveCompleted(summary)`
- `preflightFailed(reason, details)`  // e.g., insufficient disk space, path too long

## UI model contracts
- **Tree model** nodes have roles: `nodeType` = "shows|show|season|movies|movie", `key` (stable id), `title`.
- **Grid model** rows expose: `checked` (bool), `type`, `name`, `season`, `episode`, `confidence`, `dest`, `source`, `reasons`.

## Selection behavior
- Changing selection in `treeLibrary` updates `tableDetails` filter.
- Double‑click in `tableDetails` emits `openDetailsRequested(rowKey)`.
- Confidence filter buttons (All/Green/Yellow/Red) emit `filterChanged({ confidence: 'all|green|yellow|red' })`.

## Logging
- Append lines to `textActivity` via `appendActivity(message, level)`; also write to rotating log file.

## Conflicts
- On destination conflict, UI gathers a choice and emits `conflictResolutionChosen(strategy)` where `strategy ∈ {'skip','replace','keepBoth'}`.

## Notes
- `runRequested(selectedItems)` may be handled as an alias of `moveRequested(selectedItems)` for backward compatibility.
