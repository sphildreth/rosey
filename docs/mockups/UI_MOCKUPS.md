
# Rosey — UI Mockups (v1) • 2025-10-25

These low-fidelity mockups are **agent-friendly**: every control has a stable `id` you can use in code.
They map directly to PySide6 widgets (Qt Widgets), but they are toolkit-agnostic.

## Main Window (id: `mainWindow`)

```
┌───────────────────────────────────────────────────────────────────────────────────────────┐
│ Action Bar                                                                                │
│ [ Run btnRun ]  [ Configuration btnConfig ]  [ Scan btnScan ]  [ Move Selected btnMove ] │
│ [ Select All Green btnSelectGreen ]  [ Clear btnClear ]  [ Help btnHelp ]                │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Left Pane: Library Tree (treeLibrary)          │ Right Pane: Grid (tableDetails)          │
│ ┌─────────────────────────────────────────────┐ │ ┌──────────────────────────────────────┐ │
│ │ Shows (nodeShows)                           │ │ │ ✔ | Type | Name | Season | Ep | Conf│ │
│ │  ├── The Office (show:the_office)           │ │ │──┼─────┼─────┼───────┼────┼──────┤ │
│ │  │    ├── Season 01 (season:the_office_s1) │ │ │ ☐ | TV  | ... | 01    | 01 | 78    │ │
│ │  │    └── Season 02 (season:the_office_s2) │ │ │ ☐ | TV  | ... | 02    | 01 | 90    │ │
│ │  └── WKRP in Cincinnati (show:wkrp)         │ │ │ ☐ | TV  | ... | 01    | 03 | 72    │ │
│ │                                             │ │ │ ☐ | File| theme.mp3 | — | — | —     │ │
│ ├─────────────────────────────────────────────┤ │ ├──────────────────────────────────────┤ │
│ │ Movies (nodeMovies)                         │ │ │ Row details (selected item) → shown │ │
│ │  ├── Inception (2010) (movie:inception)     │ │ │ in the Details Pane / dialog        │ │
│ │  └── Big Trouble in Little China (1986)     │ │ └──────────────────────────────────────┘ │
│ └─────────────────────────────────────────────┘ │                                          │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Activity Log (textActivity) — copy-friendly                                                 │
│ [12:03:11] Scan started at C:\Downloads\Unsorted                                          │
│ [12:03:12] Found TV: The Office — S02E01 — "The Dundies" (confidence 90, TMDB match)        │
│ [12:03:13] Planned: TV/The Office/Season 02/...                                             │
│ [12:03:15] Move: source → dest (cross-volume, 734 MB, 3.2 s)                                │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Status Bar (statusBar):  Items: 23  |  Green: 8  Yellow: 9  Red: 6  |  Op: Scanning 54%    │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Interaction notes
- Selecting a node in `treeLibrary` filters `tableDetails` to that scope (show/season/movie).  
- Double‑clicking a grid row opens **Detail Dialog** (`dlgDetail`) for that media file.  
- The **Run** button executes the current plan (equivalent to “Move Selected”), but stays disabled until a plan exists.

---

## Configuration Dialog (id: `dlgConfig`)

```
┌────────────────────────── Configuration ──────────────────────────┐
│ Source Root (txtSource)     [Browse btnBrowseSource]              │
│ Movies Dir  (txtMovies)     [Browse btnBrowseMovies]              │
│ TV Dir      (txtTv)         [Browse btnBrowseTv]                  │
│                                                                    │
│ [x] Use Online Lookups (chkUseOnline)                              │
│ Provider Priority (cmbPriority1, cmbPriority2)                     │
│ TMDB API Key (txtTmdbKey)   [ Test btnTestTmdb ]                   │
│ Language (cmbLang)          Region (cmbRegion)                     │
│                                                                    │
│ Concurrency: Local (spinLocal)  Network (spinNetwork)              │
│ Cache TTL days (spinTtl)   Cache Dir (txtCacheDir) [Browse]        │
│ Theme:  (cmbTheme: system|light|dark)                              │
│                                                                    │
│ [ Save btnSaveConfig ]                     [ Cancel btnCancelCfg ] │
└────────────────────────────────────────────────────────────────────┘
```

---

## Detail Dialog (id: `dlgDetail`) — Show/Movie/Media file

```
┌────────────────────────── Details ────────────────────────────────┐
│ Title:     (lblTitle)     Type: (lblType)   Confidence: (lblConf) │
│ Source:    (lblSourcePath)                                            │
│ Destination (lblDestPath)                                             │
│ Reasons:   (listReasons)                                              │
│ Online:    Provider (lblProvider), ID (lblProviderId)                 │
│                                                                        │
│ [ Preview pane (stackPreview) ]                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ If video: QVideoWidget/ffprobe snapshot (videoPreview)           │  │
│  │ If image: QLabel (artPreview)                                    │  │
│  │ If NFO: QTextEdit read‑only                                      │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│ [ Open Source btnOpenSrc ]  [ Reveal Dest btnRevealDest ]  [ Close ]   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Context Menu(s)
- **treeLibrary**: *Scan Node*, *Expand All*, *Collapse All*, *Open in File Manager*.
- **tableDetails** (row): *Open Details*, *Open Source*, *Reveal Destination*, *Include/Exclude from Plan*.

---

## Keyboard Shortcuts
- `F5` Scan (`btnScan`), `Ctrl+Enter` Run (`btnRun`), `Ctrl+,` Configuration (`btnConfig`)
- `Ctrl+G` Select All Green, `Ctrl+L` focus Activity log, `Esc` close dialogs

---

## Theming & Layout
- Provide QSS files: `assets/light.qss`, `assets/dark.qss`.  
- Main split: `QSplitter` horizontal (tree left, grid right).  
- Bottom `QSplitter` vertical for Activity Log.  
- Status bar shows operation + progress bar + error badge.

