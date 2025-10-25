# Rosey Setup Guide

Welcome to Rosey! This guide will help you get started with installing and configuring Rosey for your media library.

## System Requirements

- **Operating System:** Windows 10/11 or Linux (Ubuntu 20.04+, Debian, Fedora, etc.)
- **Display:** GUI environment (X11 on Linux)
- **Disk Space:** ~100 MB for application, plus space for your media library
- **Network:** Optional (for online metadata lookups via TMDB/TVDB)

## Installation

### Windows

1. Download `rosey-windows.zip` from the releases page
2. Extract the ZIP file to a folder (e.g., `C:\Program Files\Rosey`)
3. Run `rosey.exe`

**Note:** Windows may show a SmartScreen warning. Click "More info" → "Run anyway" (the app is not yet code-signed).

### Linux

1. Download `rosey-linux.tar.gz` from the releases page
2. Extract: `tar -xzf rosey-linux.tar.gz`
3. Run: `./rosey/rosey`

**Optional:** Create a desktop shortcut:
```bash
# Copy icon
cp rosey/graphics/rosey_256.png ~/.local/share/icons/rosey.png

# Create desktop entry
cat > ~/.local/share/applications/rosey.desktop <<EOF
[Desktop Entry]
Type=Application
Name=Rosey
Comment=Media organizer for Jellyfin
Exec=/path/to/rosey/rosey
Icon=rosey
Categories=Utility;FileTools;
EOF
```

## First Launch

When you first launch Rosey:

1. Click **Settings** (⚙️ icon) to open configuration
2. Set your paths:
   - **Source:** Folder(s) to scan (can be network shares)
   - **Movies Destination:** Where organized movies go
   - **TV Shows Destination:** Where organized TV shows go

3. **(Optional)** Configure online lookups:
   - Toggle "Enable Online Providers"
   - Add TMDB API key (free at https://www.themoviedb.org/settings/api)
   - Add TVDB API key if needed (optional)

4. Click **Save**

## Basic Workflow

### 1. Scan Your Media

- Click **Scan** button
- Rosey will recursively scan your source folder(s)
- Files are identified using:
  - Filenames (e.g., `Movie.Name.2020.1080p.mkv`)
  - Folder names (e.g., `Show Name/Season 01/`)
  - `.nfo` files (Jellyfin/Kodi metadata)
  - Online lookups (if enabled)

### 2. Review Results

- **Green items** (confidence ≥70%): High confidence matches
- **Yellow items** (40-69%): Medium confidence, review suggested
- **Red items** (<40%): Low confidence, manual verification needed

Click on any item to see:
- Confidence score and reasons
- Proposed destination path
- Metadata (title, year, season/episode)

### 3. Select Items to Organize

- Click **Select All Green** to auto-select high-confidence items
- Manually check/uncheck items as needed
- Use filters to narrow the view (Movies/TV/Confidence)

### 4. Execute Move (Dry-Run First!)

**Important:** Dry-run mode is enabled by default for safety.

- Click **Move Selected**
- Review the preview of what will happen
- If satisfied, disable dry-run in Settings and run again
- Rosey will:
  - Move files to correct Jellyfin structure
  - Co-move sidecars (.srt, .nfo, artwork)
  - Handle conflicts (Skip/Replace/Keep Both)
  - Provide rollback on failures

### 5. Monitor Progress

- Watch the **Activity Log** at the bottom
- Progress dialog shows per-file status
- Cancel anytime (safe rollback)

## Configuration Options

### Settings Dialog (⚙️)

- **Paths:**
  - Source folder(s)
  - Movies destination
  - TV Shows destination

- **Behavior:**
  - Dry-run mode (recommended: ON for first runs)
  - File handling on conflicts

- **Online Providers:**
  - Enable/disable TMDB/TVDB lookups
  - API keys
  - Language/region preferences
  - Cache TTL

- **Advanced:**
  - Concurrency (scan threads)
  - Log level

- **Appearance:**
  - Theme: System/Light/Dark

### Configuration File

Settings are saved to `rosey.json` in:
- **Windows:** `%APPDATA%\Rosey\rosey.json`
- **Linux:** `~/.config/rosey/rosey.json`

You can edit this file directly, but use the Settings dialog when possible.

## Naming Conventions

Rosey organizes media following Jellyfin's recommended structure:

### Movies
```
Movies/
  Movie Name (2020)/
    Movie Name (2020).mkv
    Movie Name (2020).srt
    Movie Name (2020).nfo
```

### TV Shows
```
TV Shows/
  Show Name/
    Season 01/
      Show Name - S01E01 - Episode Title.mkv
      Show Name - S01E01 - Episode Title.srt
    Season 02/
      ...
```

### Specials
Episodes with season 0 are placed in `Season 00/` (Jellyfin convention).

### Multi-Episode Files
Files containing multiple episodes: `Show Name - S01E01-E02 - Episode Title.mkv`

### Multi-Part Movies
Movies split across files: `Movie Name (2020) - Part 1.mkv`, `Movie Name (2020) - Part 2.mkv`

## Tips & Best Practices

### Scanning Large Libraries

- First scan can take time (thousands of files)
- Use multiple source folders if needed
- Rosey scans in background threads (UI stays responsive)

### Network Shares

Rosey supports scanning network shares (SMB/NFS/etc.):

**Windows:**
- Use UNC paths: `\\server\share\media`
- Or map network drives: `Z:\media`

**Linux:**
- Mount shares first: `/mnt/nas/media`
- Ensure read/write permissions

**Performance tip:** Network scans are slower; adjust concurrency in Settings.

### Dry-Run Mode

Always test with dry-run mode first!
- Preview destination paths
- Check for conflicts
- Verify naming looks correct
- No files are moved in dry-run

### Backups

Although Rosey is designed to be safe:
- Keep backups of your media library
- Test with a small subset first
- Use dry-run mode extensively

### Online Lookups

**When to enable:**
- Ambiguous filenames (e.g., `episode 1.mkv`)
- Missing metadata
- Better episode titles
- Artwork/descriptions

**When to skip:**
- You have comprehensive `.nfo` files
- Privacy concerns (calls to TMDB/TVDB)
- Offline environment
- Rate limits (free TMDB key: 40 requests/10 seconds)

### File Permissions

Rosey needs:
- **Read** access to source folders
- **Write** access to destination folders

On Linux, check permissions:
```bash
ls -ld /path/to/source
ls -ld /path/to/destination
```

## Troubleshooting

### "Cannot scan source folder"

**Cause:** Permission denied or path doesn't exist

**Solution:**
- Verify path exists
- Check read permissions
- For network shares: ensure mounted and accessible

### "Move failed: destination conflict"

**Cause:** File already exists at destination

**Solution:**
- Review the conflict dialog
- Choose: Skip / Replace / Keep Both
- Keep Both adds suffix: `filename (1).mkv`

### "API rate limit exceeded"

**Cause:** Too many TMDB/TVDB requests

**Solution:**
- Wait for rate limit to reset (typically 10 seconds)
- Reduce concurrency in Settings
- Use cache (enabled by default)
- Offline mode: disable online providers

### "No display available" (Linux)

**Cause:** Running without X11/Wayland display

**Solution:**
- Ensure GUI environment is running
- For SSH: `export DISPLAY=:0` (if X11 forwarding enabled)
- For headless servers: Rosey requires a display (no CLI-only mode yet)

### App won't start after update

**Solution:**
- Clear config cache: delete `rosey.json`
- Re-extract the archive (don't overwrite old version)
- Check system requirements

### Long paths (Windows)

**Issue:** Windows has 260-character path limit

**Solution:**
- Enable long path support (Windows 10 1607+):
  - Registry: `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\FileSystem`
  - Set `LongPathsEnabled` to 1
- Or use shorter destination paths

### Missing icons/theme

**Cause:** Graphics not bundled correctly

**Solution:**
- Re-download the release
- Ensure `graphics/` folder is in same directory as binary

## Logs

Rosey writes detailed logs to:
- **Windows:** `%APPDATA%\Rosey\logs\`
- **Linux:** `~/.config/rosey/logs/`

**Log files:**
- `rosey.log` - rotating log (10 MB max per file, 5 files kept)
- Errors, warnings, and debug info

**Note:** API keys and sensitive info are redacted from logs.

## Getting Help

- Check this guide first
- Review logs for error details
- Search GitHub issues
- Open a new issue with:
  - Rosey version
  - Operating system
  - Steps to reproduce
  - Relevant log excerpts (redact personal info!)

## Privacy & Security

- **No telemetry:** Rosey does not send usage data anywhere
- **API keys:** Stored locally in `rosey.json`, never transmitted except to TMDB/TVDB
- **Online lookups:** Opt-in only; disabled by default
- **Logs:** Kept locally, API keys redacted

## What's Next?

After organizing your media:
1. Point Jellyfin to your organized Movies/TV Shows folders
2. Scan libraries in Jellyfin
3. Enjoy your perfectly organized media! 🎬📺

For more details, see:
- [Product Requirements (PRD)](./PRD.md)
- [Technical Specification](./TECH_SPEC.md)
- [Development Guide](./IMPLEMENTATION_GUIDE.md)
