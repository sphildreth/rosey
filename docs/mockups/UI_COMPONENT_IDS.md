
# Component IDs and Types

| id                   | Qt Type             | Notes |
|----------------------|---------------------|-------|
| mainWindow           | QMainWindow         | Root window |
| btnScan              | QPushButton         | Starts scan task |
| btnMove              | QPushButton         | Applies plan for checked rows (Move Selected) |
| btnSelectGreen       | QPushButton         | Checks all Green rows |
| btnClearView         | QPushButton         | Clears tree/grid/log view state |
| btnHelp              | QPushButton         | Opens README/about |
| btnConfig            | QPushButton         | Opens dlgConfig |
| btnFilterAll         | QPushButton         | Confidence filter: All |
| btnFilterGreen       | QPushButton         | Confidence filter: Green |
| btnFilterYellow      | QPushButton         | Confidence filter: Yellow |
| btnFilterRed         | QPushButton         | Confidence filter: Red |
| treeLibrary          | QTreeView           | Model groups: Shows/Seasons/Movies |
| tableDetails         | QTableView          | Backed by QAbstractTableModel |
| textActivity         | QTextEdit           | Read-only, mono, copy-friendly |
| statusBar            | QStatusBar          | Progress + counts |
| dlgConfig            | QDialog             | Configuration dialog |
| dlgDetail            | QDialog             | Details view with preview |
| btnOpenSrc           | QPushButton         | Open source path in file manager |
| btnRevealDest        | QPushButton         | Reveal destination path |
| videoPreview         | QWidget placeholder | Integrate QMediaPlayer or external preview |
| artPreview           | QLabel              | Pixmap preview |
| listReasons          | QListWidget         | Reasons for confidence |
| ctxScanNode          | QAction             | Tree: Scan Node |
| ctxExpandAll         | QAction             | Tree: Expand All |
| ctxCollapseAll       | QAction             | Tree: Collapse All |
| ctxOpenInFileManager | QAction             | Tree: Open in File Manager |
| ctxOpenDetails       | QAction             | Table: Open Details |
| ctxOpenSource        | QAction             | Table: Open Source |
| ctxRevealDestination | QAction             | Table: Reveal Destination |
| ctxToggleInclude     | QAction             | Table: Include/Exclude from Plan |
