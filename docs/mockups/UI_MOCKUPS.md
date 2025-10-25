
# Rosey — UI Mockups (v1) • 2025-10-25

These low-fidelity mockups are **agent-friendly**: every control has a stable `id` you can use in code.
They map directly to PySide6 widgets (Qt Widgets), but they are toolkit-agnostic.

## Main Window (id: `mainWindow`)

```
┌───────────────────────────────────────────────────────────────────────────────────────────┐
│ Action Bar                                                                                │
│ [ Scan btnScan ] [ Move Selected btnMove ] │ [Filter: All btnFilterAll] [Green btnFilterGreen] [Yellow btnFilterYellow] [Red btnFilterRed] │ [ Configuration btnConfig ] │
│ [ Select All Green btnSelectGreen ]  [ Clear View btnClearView ]  [ Help btnHelp ]                │
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
- The filter buttons (`btnFilter*`) update `tableDetails` to show items of a specific confidence level.
- `btnClearView` clears the `treeLibrary` and `tableDetails` views, but does not delete any data.
- Double‑clicking a grid row opens **Detail Dialog** (`dlgDetail`) for that media file.  

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
│ TVDB API Key (txtTvdbKey)   [ Test btnTestTvdb ]                   │
│ Language (cmbLang)          Region (cmbRegion)                     │
│                                                                    │
│ [ ] Dry Run (Preview changes without moving files) (chkDryRun)     │
│                                                                    │
│ Concurrency: Local (spinLocal)  Network (spinNetwork)              │
│ Cache TTL days (spinTtl)   Cache Dir (txtCacheDir) [Browse]        │
│ Theme:  (cmbTheme: system|light|dark)                              │
│                                                                    │
│ [ Save btnSaveConfig ]                     [ Cancel btnCancelCfg ] │
└────────────────────────────────────────────────────────────────────┘

### Interaction notes
- API keys are stored securely in the system keychain, not the JSON config. The text box will show a placeholder like `********` if a key is present.
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

## Error States Wireframes

### Scan Failure (Permission Denied)
```
┌───────────────────────────────────────────────────────────────────────────────────────────┐
│ Action Bar                                                                                │
│ [ Scan btnScan ] [ Move Selected btnMove ] │ [Filter: All btnFilterAll] ...                │
│ [ Select All Green btnSelectGreen ]  [ Clear View btnClearView ]  [ Help btnHelp ]        │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Left Pane: Library Tree (treeLibrary)          │ Right Pane: Grid (tableDetails)          │
│ ┌─────────────────────────────────────────────┐ │ ┌──────────────────────────────────────┐ │
│ │ (Empty - scan failed)                       │ │ │ (Empty)                               │ │
│ └─────────────────────────────────────────────┘ │ └──────────────────────────────────────┘ │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Activity Log (textActivity) — copy-friendly                                                 │
│ [12:03:11] Scan failed: Permission denied on /mnt/network/share                           │
│ [12:03:12] Error: Access denied. Check permissions or try different source.               │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Status Bar (statusBar):  Items: 0  |  Green: 0  Yellow: 0  Red: 0  |  Op: Error (🔴)       │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Move Conflict (File Exists)
```
┌───────────────────────────────────────────────────────────────────────────────────────────┐
│ ... (same as main window) ...                                                              │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Activity Log (textActivity)                                                                │
│ [12:05:15] Move: Conflict at Movies/Inception (2010)/Inception (2010).mkv - file exists    │
│ [12:05:16] Options: Skip / Replace / Keep Both (will rename to (1))                        │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Status Bar:  Items: 23  |  Green: 8  Yellow: 9  Red: 6  |  Op: Paused (Conflict)          │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Network Disconnection During Move
```
┌───────────────────────────────────────────────────────────────────────────────────────────┐
│ ...                                                                                       │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Activity Log (textActivity)                                                                │
│ [12:10:20] Move: Network share disconnected. Retrying in 2s...                             │
│ [12:10:22] Move: Retrying... (attempt 2/3)                                                 │
│ [12:10:25] Move: Failed after 3 retries. Rolled back partial moves.                        │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ Status Bar:  Items: 23  |  Green: 8  Yellow: 9  Red: 6  |  Op: Error (🔴)                 │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

---

