use camino::Utf8Path;
use rosey_core::*;
use std::fs;

fn make_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
}

fn media_path(dir: &std::path::Path, name: &str) -> camino::Utf8PathBuf {
    Utf8Path::from_path(&dir.join(name)).unwrap().to_path_buf()
}

// ─────────────────────────────────────────────────────────────────────────────
// Same-directory companions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn same_directory_subtitle() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    make_file(dir, "Movie (2023).srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("Movie (2023).srt"));
}

#[test]
fn same_directory_image() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    make_file(dir, "poster.jpg", "image");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("poster.jpg"));
}

#[test]
fn same_directory_mixed() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    make_file(dir, "Movie (2023).srt", "subs");
    make_file(dir, "poster.jpg", "image");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Subtitle folder discovery
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn subs_folder_basic() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir(dir.join("Subs")).unwrap();
    make_file(dir, "Subs/en.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("Subs/en.srt"));
}

#[test]
fn subs_folder_case_insensitive() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir(dir.join("subs")).unwrap();
    make_file(dir, "subs/en.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("subs/en.srt"));
}

#[test]
fn subtitles_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir(dir.join("Subtitles")).unwrap();
    make_file(dir, "Subtitles/en.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("Subtitles/en.srt"));
}

#[test]
fn sub_folder_variant() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir(dir.join("Sub")).unwrap();
    make_file(dir, "Sub/en.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("Sub/en.srt"));
}

#[test]
fn subtitle_folder_variant() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir(dir.join("Subtitle")).unwrap();
    make_file(dir, "Subtitle/en.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("Subtitle/en.srt"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Nested subtitle folders
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn nested_subtitle_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir_all(dir.join("Subs").join("English")).unwrap();
    make_file(dir, "Subs/English/en.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("Subs/English/en.srt"));
}

#[test]
fn deeply_nested_subtitle_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir_all(dir.join("Subs").join("English").join("Forced")).unwrap();
    make_file(dir, "Subs/English/Forced/en.forced.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 1);
    assert!(companions[0].as_str().ends_with("en.forced.srt"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Multiple formats
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn multiple_subtitle_formats() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir(dir.join("Subs")).unwrap();
    make_file(dir, "Subs/en.srt", "subs");
    make_file(dir, "Subs/en.ass", "subs");
    make_file(dir, "Subs/en.vtt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 3);
}

#[test]
fn mixed_locations() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    make_file(dir, "Movie (2023).srt", "subs");
    make_file(dir, "poster.jpg", "image");
    fs::create_dir(dir.join("Subs")).unwrap();
    make_file(dir, "Subs/en.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert_eq!(companions.len(), 3);
}

// ─────────────────────────────────────────────────────────────────────────────
// Ignores non-subtitle folders
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ignores_other_folders() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    fs::create_dir(dir.join("Extras")).unwrap();
    make_file(dir, "Extras/should_not_find.srt", "subs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);

    assert!(companions.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn nonexistent_parent_returns_empty() {
    let media = Utf8Path::new("/nonexistent/path/Movie.mkv");
    let companions = discover_companion_files(media);
    assert!(companions.is_empty());
}

#[test]
fn no_extension_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    make_file(dir, "Movie (2023).mkv", "video");
    make_file(dir, "README", "docs");

    let media = media_path(dir, "Movie (2023).mkv");
    let companions = discover_companion_files(&media);
    assert!(companions.is_empty());
}
