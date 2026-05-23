use camino::{Utf8Path, Utf8PathBuf};
use std::collections::HashSet;
use std::fs;

/// Extensions considered companion files by the identifier.
///
/// Mirrors Python `_discover_companion_files`:
/// - subtitle_exts = {".srt", ".ass", ".vtt"}
/// - image_exts = {".jpg", ".png", ".jpeg"}
const COMPANION_SUBTITLE_EXTS: &[&str] = &["srt", "ass", "vtt"];
const COMPANION_IMAGE_EXTS: &[&str] = &["jpg", "png", "jpeg"];

/// Subtitle folder names matched case-insensitively.
const SUBTITLE_FOLDER_NAMES: &[&str] = &["subs", "sub", "subtitles", "subtitle"];

/// Discover companion files for a media file.
///
/// Looks in the same directory for subtitle and image files, and
/// recursively scans recognized subtitle subdirectories.
///
/// This is the **identifier-level** companion discovery (broader than
/// mover-level `discover_sidecars`).
pub fn discover_companion_files(media_path: &Utf8Path) -> Vec<Utf8PathBuf> {
    let Some(parent_dir) = media_path.parent() else {
        return Vec::new();
    };

    if !parent_dir.exists() {
        return Vec::new();
    }

    let subtitle_exts: HashSet<_> = COMPANION_SUBTITLE_EXTS.iter().copied().collect();
    let image_exts: HashSet<_> = COMPANION_IMAGE_EXTS.iter().copied().collect();
    let subtitle_folders: HashSet<_> = SUBTITLE_FOLDER_NAMES.iter().copied().collect();

    let mut companions = Vec::new();

    let Ok(entries) = fs::read_dir(parent_dir) else {
        return companions;
    };

    for entry in entries.filter_map(Result::ok) {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        let Some(path) = Utf8PathBuf::from_path_buf(entry.path()).ok() else {
            continue;
        };

        if file_type.is_file() {
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_lowercase();
                if subtitle_exts.contains(ext_lower.as_str())
                    || image_exts.contains(ext_lower.as_str())
                {
                    companions.push(path);
                }
            }
        } else if file_type.is_dir() {
            if let Some(name) = path.file_name() {
                if subtitle_folders.contains(name.to_lowercase().as_str()) {
                    // Recursively find all subtitle files in this folder
                    companions.extend(recursive_subtitle_scan(&path, &subtitle_exts));
                }
            }
        }
    }

    companions
}

fn recursive_subtitle_scan(dir: &Utf8Path, subtitle_exts: &HashSet<&str>) -> Vec<Utf8PathBuf> {
    let mut results = Vec::new();

    let Ok(entries) = fs::read_dir(dir) else {
        return results;
    };

    for entry in entries.filter_map(Result::ok) {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        let Some(path) = Utf8PathBuf::from_path_buf(entry.path()).ok() else {
            continue;
        };

        if file_type.is_file() {
            if let Some(ext) = path.extension() {
                if subtitle_exts.contains(ext.to_lowercase().as_str()) {
                    results.push(path);
                }
            }
        } else if file_type.is_dir() {
            results.extend(recursive_subtitle_scan(&path, subtitle_exts));
        }
    }

    results
}
