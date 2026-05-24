use camino::Utf8Path;
use rosey_fs::*;
use std::fs;

fn make_video(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
}

// ─────────────────────────────────────────────────────────────────────────────
// Basic scanning
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn scanner_basic() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    make_video(root, "The Matrix (1999).mkv", "fake video data");
    make_video(root, "Inception (2010).mp4", "fake video data");
    make_video(root, "Show S01E01.mkv", "fake video data");
    make_video(root, "Show S01E02.avi", "fake video data");

    let subdir = root.join("subfolder");
    fs::create_dir(&subdir).unwrap();
    make_video(&subdir, "Movie.mp4", "fake video");
    fs::write(subdir.join("document.txt"), "not a video").unwrap();

    let scanner = Scanner::new(2, false);
    let results = scanner.scan(Utf8Path::from_path(root).unwrap());

    let videos: Vec<_> = results.iter().filter(|r| r.is_video).collect();
    assert_eq!(videos.len(), 5);

    let names: std::collections::HashSet<_> =
        videos.iter().map(|r| r.path.file_name().unwrap().to_string()).collect();

    assert!(names.contains("The Matrix (1999).mkv"));
    assert!(names.contains("Inception (2010).mp4"));
    assert!(names.contains("Show S01E01.mkv"));
    assert!(names.contains("Movie.mp4"));
}

#[test]
fn scanner_video_extensions() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    make_video(root, "video.mkv", "fake");
    make_video(root, "video.mp4", "fake");
    make_video(root, "video.avi", "fake");
    fs::write(root.join("doc.txt"), "not video").unwrap();

    let scanner = Scanner::default();
    let results = scanner.scan(Utf8Path::from_path(root).unwrap());

    for r in &results {
        if r.path.extension() == Some("txt") {
            assert!(!r.is_video);
        } else {
            assert!(r.is_video);
        }
    }
}

#[test]
fn scanner_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let scanner = Scanner::default();
    let results = scanner.scan(Utf8Path::from_path(tmp.path()).unwrap());
    assert!(results.is_empty());
}

#[test]
fn scanner_nonexistent_path() {
    let scanner = Scanner::default();
    let results = scanner.scan(Utf8Path::new("/nonexistent/path/12345"));
    assert!(results.is_empty());
}

#[test]
fn scanner_single_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let video = root.join("single.mkv");
    fs::write(&video, "fake video").unwrap();

    let scanner = Scanner::default();
    let results = scanner.scan(Utf8Path::from_path(&video).unwrap());

    assert_eq!(results.len(), 1);
    assert!(results[0].is_video);
    assert_eq!(results[0].path.as_str(), video.to_str().unwrap());
}

#[test]
fn scanner_concurrency_levels_produce_same_results() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    make_video(root, "a.mkv", "1");
    make_video(root, "b.mp4", "2");
    let sub = root.join("sub");
    fs::create_dir(&sub).unwrap();
    make_video(&sub, "c.avi", "3");

    let s1 = Scanner::new(1, false);
    let s4 = Scanner::new(4, false);

    let r1 = s1.scan(Utf8Path::from_path(root).unwrap());
    let r4 = s4.scan(Utf8Path::from_path(root).unwrap());

    let paths1: std::collections::HashSet<_> = r1.iter().map(|r| r.path.clone()).collect();
    let paths4: std::collections::HashSet<_> = r4.iter().map(|r| r.path.clone()).collect();

    assert_eq!(paths1, paths4);
}

#[test]
fn scan_directory_convenience() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    make_video(root, "a.mkv", "1");
    make_video(root, "b.mp4", "2");
    let sub = root.join("sub");
    fs::create_dir(&sub).unwrap();
    make_video(&sub, "c.avi", "3");

    let results = scan_directory(Utf8Path::from_path(root).unwrap(), 2, false);
    let videos: Vec<_> = results.iter().filter(|r| r.is_video).collect();
    assert_eq!(videos.len(), 3);
}

#[test]
fn scan_with_progress_reports_each_result() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    make_video(root, "a.mkv", "1");
    make_video(root, "b.mp4", "2");
    fs::write(root.join("doc.txt"), "not video").unwrap();

    let mut seen = Vec::new();
    let results =
        scan_with_progress(Utf8Path::from_path(root).unwrap(), ScanOptions::default(), |result| {
            seen.push(result.path.clone())
        });

    assert_eq!(seen.len(), results.len());
    assert_eq!(results.len(), 2);
    assert!(seen.iter().any(|path| path.file_name() == Some("a.mkv")));
    assert!(seen.iter().any(|path| path.file_name() == Some("b.mp4")));
}

#[test]
fn scanner_large_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    for i in 0..5 {
        let season = root.join(format!("Season {:02}", i));
        fs::create_dir(&season).unwrap();
        for j in 0..10 {
            make_video(&season, &format!("Episode_{:02}.mkv", j), "fake");
        }
    }

    let scanner = Scanner::new(4, false);
    let results = scanner.scan(Utf8Path::from_path(root).unwrap());
    let videos: Vec<_> = results.iter().filter(|r| r.is_video).collect();
    assert_eq!(videos.len(), 50);
}

#[test]
fn scanner_file_sizes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    make_video(root, "a.mkv", "12345");
    make_video(root, "b.mp4", "abcdefgh");

    let scanner = Scanner::default();
    let results = scanner.scan(Utf8Path::from_path(root).unwrap());

    for r in &results {
        if r.is_video {
            assert!(r.size_bytes > 0, "expected size > 0 for {}", r.path);
        }
    }
}
