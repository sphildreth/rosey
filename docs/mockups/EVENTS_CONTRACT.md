
# Event & Signal Contract (for agents)

## Commands (triggered by buttons / menu)
- `scanRequested(sourceDir)`
- `runRequested(selectedItems)`  // execute plan for checked rows
- `openConfigRequested()`
- `selectAllGreenRequested()`
- `clearRequested()`

## Long-running tasks → signals
- `scanProgress(foundCount, currentPath)`
- `scanCompleted(treeModel, flatItems)`
- `lookupProgress(done, total)`
- `moveProgress(bytesMoved, fileIndex, totalFiles, currentSrc, currentDst)`
- `moveCompleted(summary)`

## UI model contracts
- **Tree model** nodes have roles: `nodeType` = "shows|show|season|movies|movie", `key` (stable id), `title`.
- **Grid model** rows expose: `checked` (bool), `type`, `name`, `season`, `episode`, `confidence`, `dest`, `source`, `reasons`.

## Selection behavior
- Changing selection in `treeLibrary` updates `tableDetails` filter.  
- Double‑click in `tableDetails` emits `openDetailsRequested(rowKey)`.

## Logging
- Append lines to `textActivity` via `appendActivity(message, level)`; also write to rotating log file.
