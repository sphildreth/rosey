use camino::Utf8Path;
use rosey_fs::*;
use std::fs;

fn make_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
}

// ─────────────────────────────────────────────────────────────────────────────
// is_sidecar_path
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn is_sidecar_recognizes_all_extensions() {
    let exts = ["srt", "ssa", "ass", "vtt", "sub", "idx", "sbv", "lrc", "smi", "stl", "nfo", "png"];
    for ext in &exts {
        assert!(
            is_sidecar_path(Utf8Path::new(&format!("movie.{}", ext))),
            "expected {} to be a sidecar extension",
            ext
        );
    }
}

#[test]
fn is_sidecar_case_insensitive() {
    assert!(is_sidecar_path(Utf8Path::new("movie.SRT")));
    assert!(is_sidecar_path(Utf8Path::new("movie.PnG")));
}

#[test]
fn is_sidecar_rejects_non_sidecar() {
    assert!(!is_sidecar_path(Utf8Path::new("movie.mkv")));
    assert!(!is_sidecar_path(Utf8Path::new("movie.mp4")));
    assert!(!is_sidecar_path(Utf8Path::new("movie.txt")));
    assert!(!is_sidecar_path(Utf8Path::new("movie")));
}

#[test]
fn is_sidecar_rejects_jpg_images() {
    assert!(!is_sidecar_path(Utf8Path::new("movie.jpg")));
    assert!(!is_sidecar_path(Utf8Path::new("movie.jpeg")));
    assert!(!is_sidecar_path(Utf8Path::new("movie.JPG")));
    assert!(!is_sidecar_path(Utf8Path::new("movie.JPEG")));
}

// ─────────────────────────────────────────────────────────────────────────────
// discover_sidecars
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn discover_finds_matching_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "movie.mkv", "video");
    make_file(dir, "movie.srt", "subtitles");
    make_file(dir, "movie.nfo", "metadata");
    make_file(dir, "movie.jpg", "poster");
    make_file(dir, "movie.jpeg", "poster");
    make_file(dir, "other.srt", "other");

    let media_path = dir.join("movie.mkv");
    let media = Utf8Path::from_path(&media_path).unwrap();
    let sidecars = discover_sidecars(media);

    assert_eq!(sidecars.len(), 2);
    let names: std::collections::HashSet<_> =
        sidecars.iter().map(|p| p.file_name().unwrap().to_string()).collect();
    assert!(names.contains("movie.srt"));
    assert!(names.contains("movie.nfo"));
    assert!(!names.contains("movie.jpg"));
    assert!(!names.contains("movie.jpeg"));
    assert!(!names.contains("other.srt"));
}

#[test]
fn discover_empty_when_no_matches() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "movie.mkv", "video");

    let media_path = dir.join("movie.mkv");
    let media = Utf8Path::from_path(&media_path).unwrap();
    let sidecars = discover_sidecars(media);
    assert!(sidecars.is_empty());
}

#[test]
fn discover_handles_nonexistent_path() {
    let media = Utf8Path::new("/nonexistent/path/movie.mkv");
    let sidecars = discover_sidecars(media);
    assert!(sidecars.is_empty());
}

#[test]
fn discover_does_not_include_media_file() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "movie.mkv", "video");
    make_file(dir, "movie.mp4", "alt");

    let media_path = dir.join("movie.mkv");
    let media = Utf8Path::from_path(&media_path).unwrap();
    let sidecars = discover_sidecars(media);
    // movie.mp4 has the same stem but is a video extension; it IS in the sidecar list
    // because we don't exclude video extensions from sidecar check.
    // Actually, is_sidecar_path returns false for "mp4", so it won't be included.
    assert!(!sidecars.iter().any(|p| p.file_name() == Some("movie.mkv")));
}

#[test]
fn discover_ignores_directories() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "movie.mkv", "video");

    // Create a directory with sidecar-like name
    fs::create_dir(dir.join("movie.srt")).unwrap();

    let media_path = dir.join("movie.mkv");
    let media = Utf8Path::from_path(&media_path).unwrap();
    let sidecars = discover_sidecars(media);
    assert!(sidecars.is_empty());
}

#[test]
fn discover_case_insensitive_extensions() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "movie.mkv", "video");
    make_file(dir, "movie.SRT", "subs");
    make_file(dir, "movie.JPG", "poster");
    make_file(dir, "movie.JPEG", "poster");

    let media_path = dir.join("movie.mkv");
    let media = Utf8Path::from_path(&media_path).unwrap();
    let sidecars = discover_sidecars(media);
    assert_eq!(sidecars.len(), 1);
    assert!(sidecars.iter().any(|path| path.file_name() == Some("movie.SRT")));
}

#[test]
fn discover_different_stem_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "movie.mkv", "video");
    make_file(dir, "movie.en.srt", "subs");
    make_file(dir, "other.srt", "other");

    let media_path = dir.join("movie.mkv");
    let media = Utf8Path::from_path(&media_path).unwrap();
    let sidecars = discover_sidecars(media);
    // "movie.en.srt" has stem "movie.en", not "movie"
    assert!(sidecars.is_empty());
}

#[cfg(unix)]
mod symlink_tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn discover_includes_symlinked_sidecar_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        make_file(dir, "movie.mkv", "video");
        make_file(dir, "actual_subtitle.srt", "subs");
        symlink(dir.join("actual_subtitle.srt"), dir.join("movie.srt")).unwrap();

        let media_path = dir.join("movie.mkv");
        let media = Utf8Path::from_path(&media_path).unwrap();
        let sidecars = discover_sidecars(media);

        assert_eq!(sidecars.len(), 1);
        assert!(sidecars.iter().any(|path| path.file_name() == Some("movie.srt")));
    }
}
