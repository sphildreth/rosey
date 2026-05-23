use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
    #[serde(default = "default_max_workers")]
    pub max_workers: usize,
}

fn default_max_workers() -> usize {
    8
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self { follow_symlinks: false, max_depth: None, max_workers: default_max_workers() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub path: Utf8PathBuf,
    pub is_video: bool,
    pub size_bytes: u64,
    pub error: Option<String>,
}

pub const VIDEO_EXTENSIONS: &[&str] =
    &["mkv", "mp4", "avi", "mov", "wmv", "flv", "m4v", "mpg", "mpeg", "webm", "ts"];

pub fn is_video_path(path: &Utf8Path) -> bool {
    path.extension()
        .map(|ext| VIDEO_EXTENSIONS.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)))
        .unwrap_or(false)
}

/// Scan a filesystem path for video files.
///
/// If `root` does not exist, returns an empty vector (matching Python behavior).
/// If `root` is a single file, it is scanned directly.
pub fn scan(root: &Utf8Path, options: ScanOptions) -> Vec<ScanResult> {
    if !root.exists() {
        return Vec::new();
    }

    // Single file path
    if !root.is_dir() {
        let is_video = is_video_path(root);
        let size_bytes = if is_video {
            std::fs::metadata(root.as_std_path()).map(|m| m.len()).unwrap_or_default()
        } else {
            0
        };
        return vec![ScanResult { path: root.to_path_buf(), is_video, size_bytes, error: None }];
    }

    let mut walker = WalkDir::new(root).follow_links(options.follow_symlinks);

    if let Some(depth) = options.max_depth {
        walker = walker.max_depth(depth);
    }

    walker
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) if entry.file_type().is_file() => {
                let path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).ok()?;
                if !is_video_path(&path) {
                    return None;
                }

                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or_default();
                Some(ScanResult { path, is_video: true, size_bytes, error: None })
            }
            Ok(_) => None,
            Err(error) => {
                let path = error
                    .path()
                    .and_then(|p| Utf8PathBuf::from_path_buf(p.to_path_buf()).ok())
                    .unwrap_or_else(|| Utf8PathBuf::from(""));
                Some(ScanResult {
                    path,
                    is_video: false,
                    size_bytes: 0,
                    error: Some(error.to_string()),
                })
            }
        })
        .collect()
}

/// Scans filesystem for media files.
///
/// Mirrors the Python `Scanner` API. `max_workers` is stored for API parity;
/// the Rust implementation uses `walkdir` which handles filesystem traversal
/// efficiently without a thread pool.
pub struct Scanner {
    pub max_workers: usize,
    pub follow_symlinks: bool,
}

impl Scanner {
    /// Create a new scanner.
    pub fn new(max_workers: usize, follow_symlinks: bool) -> Self {
        Self { max_workers, follow_symlinks }
    }

    /// Scan a directory tree (or single file) for video files.
    pub fn scan(&self, root: &Utf8Path) -> Vec<ScanResult> {
        scan(
            root,
            ScanOptions {
                follow_symlinks: self.follow_symlinks,
                max_depth: None,
                max_workers: self.max_workers,
            },
        )
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new(8, false)
    }
}

/// Convenience function to scan a directory (or single file).
pub fn scan_directory(
    root: &Utf8Path,
    max_workers: usize,
    follow_symlinks: bool,
) -> Vec<ScanResult> {
    let scanner = Scanner::new(max_workers, follow_symlinks);
    scanner.scan(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_video_extensions_case_insensitively() {
        assert!(is_video_path(Utf8Path::new("Movie.MKV")));
        assert!(is_video_path(Utf8Path::new("Movie.mp4")));
        assert!(!is_video_path(Utf8Path::new("poster.jpg")));
    }
}
