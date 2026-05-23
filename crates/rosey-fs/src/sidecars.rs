use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

pub const SIDECAR_EXTENSIONS: &[&str] = &[
    "srt", "ssa", "ass", "vtt", "sub", "idx", "sbv", "lrc", "smi", "stl", "nfo", "jpg", "jpeg",
    "png",
];

pub fn is_sidecar_path(path: &Utf8Path) -> bool {
    path.extension()
        .map(|ext| SIDECAR_EXTENSIONS.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)))
        .unwrap_or(false)
}

/// Discover sidecar files that share the same base filename in the same directory.
///
/// Mirrors Python `discover_sidecars` from `rosey.mover.mover`.
pub fn discover_sidecars(media_path: &Utf8Path) -> Vec<Utf8PathBuf> {
    let Some(parent) = media_path.parent() else {
        return Vec::new();
    };
    let Some(stem) = media_path.file_stem() else {
        return Vec::new();
    };

    let Ok(entries) = fs::read_dir(parent) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .filter_map(|entry| Utf8PathBuf::from_path_buf(entry.path()).ok())
        .filter(|candidate| candidate != media_path)
        .filter(|candidate| candidate.file_stem() == Some(stem))
        .filter(|candidate| is_sidecar_path(candidate))
        .collect()
}
