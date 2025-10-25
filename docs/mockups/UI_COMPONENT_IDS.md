
# Component IDs and Types

| id                | Qt Type             | Notes |
|-------------------|---------------------|-------|
| mainWindow        | QMainWindow         | Root window |
| btnRun            | QPushButton         | Executes current plan (same as Move Selected) |
| btnConfig         | QPushButton         | Opens dlgConfig |
| btnScan           | QPushButton         | Starts scan task |
| btnMove           | QPushButton         | Applies plan for checked rows |
| btnSelectGreen    | QPushButton         | Checks all Green rows |
| btnClear          | QPushButton         | Clears grid/log |
| btnHelp           | QPushButton         | Opens README/about |
| treeLibrary       | QTreeView           | Model groups: Shows/Seasons/Movies |
| tableDetails      | QTableView          | Backed by QAbstractTableModel |
| textActivity      | QTextEdit           | Read-only, mono, copy-friendly |
| statusBar         | QStatusBar          | Progress + counts |
| dlgConfig         | QDialog             | Configuration dialog |
| dlgDetail         | QDialog             | Details view with preview |
| videoPreview      | QWidget placeholder | Integrate QMediaPlayer or external preview |
| artPreview        | QLabel              | Pixmap preview |
| listReasons       | QListWidget         | Reasons for confidence |
