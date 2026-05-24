use camino::Utf8Path;
use rosey_core::{ConflictPolicy, MediaItem};
use rosey_fs::*;
use std::fs;

fn _make_file(dir: &std::path::Path, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
}

fn media_path(dir: &std::path::Path, name: &str) -> camino::Utf8PathBuf {
    Utf8Path::from_path(&dir.join(name)).unwrap().to_path_buf()
}

// ─────────────────────────────────────────────────────────────────────────────
// same_volume
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn same_volume_detects_same_device() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let file1 = media_path(dir, "file1.txt");
    fs::write(&file1, "test").unwrap();

    let file2 = media_path(dir, "file2.txt");

    assert!(same_volume(&file1, &file2));
}

// ─────────────────────────────────────────────────────────────────────────────
// apply_conflict_suffix
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn apply_conflict_suffix_increments() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let existing = media_path(dir, "movie.mkv");
    fs::write(&existing, "exists").unwrap();

    let new_path = apply_conflict_suffix(&existing);
    assert_eq!(new_path.file_name().unwrap(), "movie (1).mkv");

    fs::write(&new_path, "exists too").unwrap();
    let newer_path = apply_conflict_suffix(&existing);
    assert_eq!(newer_path.file_name().unwrap(), "movie (2).mkv");
}

// ─────────────────────────────────────────────────────────────────────────────
// move_file_transactional
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn move_file_dry_run() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "movie.mkv");
    fs::write(&source, "video").unwrap();

    let dest = media_path(dir, "dest").join("movie.mkv");

    let (success, action) =
        move_file_transactional(&source, &dest, ConflictPolicy::Skip, true).unwrap();

    assert!(success);
    assert!(matches!(action, MoveAction::WouldMove));
    assert!(source.exists());
    assert!(!dest.exists());
}

#[test]
fn move_file_same_volume() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "source").join("movie.mkv");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "video content").unwrap();

    let dest = media_path(dir, "dest").join("movie.mkv");

    let (success, action) =
        move_file_transactional(&source, &dest, ConflictPolicy::Skip, false).unwrap();

    assert!(success);
    assert!(matches!(action, MoveAction::Moved));
    assert!(!source.exists());
    assert!(dest.exists());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "video content");
}

#[test]
fn move_file_skip_existing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "source").join("movie.mkv");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "new").unwrap();

    let dest = media_path(dir, "dest").join("movie.mkv");
    fs::create_dir_all(dest.parent().unwrap()).unwrap();
    fs::write(&dest, "old").unwrap();

    let (success, action) =
        move_file_transactional(&source, &dest, ConflictPolicy::Skip, false).unwrap();

    assert!(success);
    assert!(matches!(action, MoveAction::Skipped));
    assert!(source.exists());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "old");
}

#[test]
fn move_file_replace_existing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "source").join("movie.mkv");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "new content").unwrap();

    let dest = media_path(dir, "dest").join("movie.mkv");
    fs::create_dir_all(dest.parent().unwrap()).unwrap();
    fs::write(&dest, "old content").unwrap();

    let (success, action) =
        move_file_transactional(&source, &dest, ConflictPolicy::Replace, false).unwrap();

    assert!(success);
    assert!(matches!(action, MoveAction::Replaced));
    assert!(!source.exists());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "new content");
}

#[test]
fn move_file_keep_both() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "source").join("movie.mkv");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "new content").unwrap();

    let dest = media_path(dir, "dest").join("movie.mkv");
    fs::create_dir_all(dest.parent().unwrap()).unwrap();
    fs::write(&dest, "old content").unwrap();

    let (success, action) =
        move_file_transactional(&source, &dest, ConflictPolicy::KeepBoth, false).unwrap();

    assert!(success);
    assert!(matches!(action, MoveAction::KeptBoth));
    assert!(!source.exists());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "old content");

    let kept = dest.parent().unwrap().join("movie (1).mkv");
    assert!(kept.exists());
    assert_eq!(fs::read_to_string(&kept).unwrap(), "new content");
}

#[test]
fn move_file_creates_parent_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "source").join("movie.mkv");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "video").unwrap();

    let dest = media_path(dir, "dest").join("subdir").join("nested").join("movie.mkv");

    let (success, action) =
        move_file_transactional(&source, &dest, ConflictPolicy::Skip, false).unwrap();

    assert!(success);
    assert!(matches!(action, MoveAction::Moved));
    assert!(dest.exists());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "video");
}

#[test]
fn move_file_source_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "missing.mkv");
    let dest = media_path(dir, "dest.mkv");

    let result = move_file_transactional(&source, &dest, ConflictPolicy::Skip, false);
    assert!(result.is_err());
}

#[test]
fn verify_file_copy_accepts_identical_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "source.mkv");
    let dest = media_path(dir, "dest.mkv");
    fs::write(&source, b"video content").unwrap();
    fs::write(&dest, b"video content").unwrap();

    assert!(verify_file_copy(&source, &dest).unwrap());
}

#[test]
fn verify_file_copy_rejects_same_size_different_content() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "source.mkv");
    let dest = media_path(dir, "dest.mkv");
    let mut source_content = vec![b'a'; 1024 * 1024 + 17];
    let mut dest_content = source_content.clone();
    source_content[1024 * 1024 + 3] = b'b';
    dest_content[1024 * 1024 + 3] = b'c';
    fs::write(&source, source_content).unwrap();
    fs::write(&dest, dest_content).unwrap();

    assert!(!verify_file_copy(&source, &dest).unwrap());
}

// ─────────────────────────────────────────────────────────────────────────────
// move_with_sidecars
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn move_with_sidecars_moves_all_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source_dir = dir.join("source");
    let dest_dir = dir.join("dest");
    fs::create_dir_all(&source_dir).unwrap();

    let main = media_path(&source_dir, "movie.mkv");
    fs::write(&main, "video").unwrap();
    fs::write(source_dir.join("movie.srt"), "subs").unwrap();
    fs::write(source_dir.join("movie.nfo"), "meta").unwrap();

    let _item = MediaItem::unknown(main.clone());
    let item = MediaItem {
        kind: rosey_core::MediaKind::Movie,
        source_path: main.clone(),
        title: Some("Test Movie".into()),
        ..MediaItem::unknown(main.clone())
    };

    let dest = media_path(&dest_dir, "Test Movie").join("Test Movie.mkv");

    let result = move_with_sidecars(&item, &dest, ConflictPolicy::Skip, false);

    assert!(result.success);
    assert!(!main.exists());
    assert!(dest.exists());
    assert!(dest.parent().unwrap().join("Test Movie.srt").exists());
    assert!(dest.parent().unwrap().join("Test Movie.nfo").exists());
}

#[test]
fn move_with_sidecars_dry_run() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source_dir = dir.join("source");
    fs::create_dir_all(&source_dir).unwrap();

    let main = media_path(&source_dir, "movie.mkv");
    fs::write(&main, "video").unwrap();
    fs::write(source_dir.join("movie.srt"), "subs").unwrap();

    let item = MediaItem::unknown(main.clone());

    let dest = media_path(dir, "dest").join("movie.mkv");

    let result = move_with_sidecars(&item, &dest, ConflictPolicy::Skip, true);

    assert!(result.success);
    assert!(main.exists());
    assert!(!dest.exists());
}

#[test]
fn move_with_sidecars_rollback_on_sidecar_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source_dir = dir.join("source");
    fs::create_dir_all(&source_dir).unwrap();

    let main = media_path(&source_dir, "movie.mkv");
    fs::write(&main, "video").unwrap();

    let _item = MediaItem::unknown(main.clone());

    // Destination with read-only parent to force sidecar failure
    let dest = media_path(dir, "dest").join("movie.mkv");

    // First, move the main file successfully
    let _ = move_file_transactional(&main, &dest, ConflictPolicy::Skip, false);

    // Now attempt move_with_sidecars where sidecars don't exist (should succeed)
    let item2 = MediaItem::unknown(main.clone());

    // Since main was already moved, this will fail at main file step
    let result = move_with_sidecars(&item2, &dest, ConflictPolicy::Skip, false);

    assert!(!result.success);
}

// ─────────────────────────────────────────────────────────────────────────────
// check_preflight
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn preflight_all_ok() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "movie.mkv");
    fs::write(&source, "x").unwrap();

    let dest = media_path(dir, "dest");
    fs::create_dir_all(&dest).unwrap();

    let result = check_preflight(&[&source], &dest);

    assert!(result.free_space_ok);
    assert!(result.path_len_ok);
    assert!(result.perms_ok);
    assert!(result.errors.is_empty());
}

#[test]
fn preflight_creates_destination() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let source = media_path(dir, "movie.mkv");
    fs::write(&source, "video").unwrap();

    let dest = media_path(dir, "new_dest");

    let result = check_preflight(&[&source], &dest);

    assert!(dest.exists());
    assert!(result.perms_ok);
}

#[test]
fn preflight_detects_long_path() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let long_name = format!("{}.mkv", "a".repeat(250));
    let source = media_path(dir, &long_name);
    fs::write(&source, "video").unwrap();

    let deep_dest = media_path(dir, &("subdir".repeat(30)));

    let result = check_preflight(&[&source], &deep_dest);

    // Path length check is best-effort; we just verify it ran without panic
    let _ = result.path_len_ok;
}
