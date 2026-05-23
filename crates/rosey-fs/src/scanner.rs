use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub follow_symlinks: bool,
    pub max_depth: Option<usize>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            follow_symlinks: false,
            max_depth: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub path: Utf8PathBuf,
    pub is_video: bool,
    pub size_bytes: u64,
    pub error: Option<String>,
}

pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "avi", "mov", "wmv", "flv", "m4v", "mpg", "mpeg", "webm", "ts",
];

pub fn is_video_path(path: &Utf8Path) -> bool {
    path.extension()
        .map(|ext| VIDEO_EXTENSIONS.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)))
        .unwrap_or(false)
}

pub fn scan(root: &Utf8Path, options: ScanOptions) -> Vec<ScanResult> {
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
                Some(ScanResult {
                    path,
                    is_video: true,
                    size_bytes,
                    error: None,
                })
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
