use camino::Utf8PathBuf;
use rosey_core::{identify_file_fast, identify_file_with_config, MediaKind, RoseyConfig};

#[test]
fn identify_file_uses_file_stem_for_movie_titles() {
    let item = identify_file_fast(
        &Utf8PathBuf::from("/media/Some.Movie.2024.mkv"),
        &RoseyConfig::default(),
    )
    .item;

    assert_eq!(item.kind, MediaKind::Movie);
    assert_eq!(item.year, Some(2024));
    assert_eq!(item.title.as_deref(), Some("Some Movie"));
    assert!(!item.title.as_deref().unwrap().contains("mkv"));
}

#[test]
fn path_tmdb_id_extracts_from_directory_name() {
    let mut config = RoseyConfig::default();
    config.identification.movies_always_in_own_directory = false;

    let result = identify_file_fast(
        &Utf8PathBuf::from("/movies/The Matrix (1999) [tmdbid-603]/The Matrix.mkv"),
        &config,
    );

    assert_eq!(result.item.kind, MediaKind::Movie);
    assert_eq!(result.item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("603"));
    assert!(result.reasons.iter().any(|r| r.contains("TMDB ID from directory name")));
}

#[test]
fn path_imdb_id_extracts_from_directory_name() {
    let config = RoseyConfig::default();

    let result = identify_file_fast(
        &Utf8PathBuf::from("/tv/Breaking Bad (2008) [imdbid-tt0903747]/Season 01/S01E01.mkv"),
        &config,
    );

    assert_eq!(result.item.nfo.get("imdbid").and_then(|id| id.as_deref()), Some("tt0903747"));
}

#[test]
fn movie_titles_drop_release_tags_and_technical_segments() {
    let examples = [
        (
            "/media/Good.Luck.Have.Fun.Dont.Die.2025.1080p.WEBRip.x265.10bit.AAC5.1-[YTS.BZ].mp4",
            "Good Luck Have Fun Dont Die",
            2025,
        ),
        (
            "/media/Problemista.2023.1080p.WEBRip.x265.10bit.AAC5.1-[YTS.MX].mp4",
            "Problemista",
            2023,
        ),
        (
            "/media/Stolen.Assignment.1955.1080p.BluRay.x264.AAC-[YTS.BZ].mp4",
            "Stolen Assignment",
            1955,
        ),
        (
            "/media/Project Hail Mary (2026) [IMAX] [1080p] [WEBRip] [x265] [10bit] [5_1] [YTS_BZ].mp4",
            "Project Hail Mary",
            2026,
        ),
    ];

    for (path, title, year) in examples {
        let item = identify_file_fast(&Utf8PathBuf::from(path), &RoseyConfig::default()).item;

        assert_eq!(item.kind, MediaKind::Movie, "{path}");
        assert_eq!(item.title.as_deref(), Some(title), "{path}");
        assert_eq!(item.year, Some(year), "{path}");
    }
}

#[test]
fn movie_identification_falls_back_to_directory_when_filename_lacks_year() {
    let item = identify_file_fast(
        &Utf8PathBuf::from(
            "/incoming/Carry On Regardless (1961) [1080p] [WEBRip] [YTS.MX]/Carry.On.Regardless.1080p.WEBRip.x264.AAC-[YTS.MX].mp4",
        ),
        &RoseyConfig::default(),
    )
    .item;

    assert_eq!(item.kind, MediaKind::Movie);
    assert_eq!(item.title.as_deref(), Some("Carry On Regardless"));
    assert_eq!(item.year, Some(1961));
}

#[test]
fn configured_title_remove_segments_are_removed_case_insensitively() {
    let mut config = RoseyConfig::default();
    config.identification.title_remove_segments = vec!["fan edit".into()];

    let item = identify_file_fast(
        &Utf8PathBuf::from("/media/Example.Movie.Fan.Edit.2024.1080p.WEBRip.mkv"),
        &config,
    )
    .item;

    assert_eq!(item.title.as_deref(), Some("Example Movie"));
    assert_eq!(item.year, Some(2024));
}

#[cfg(unix)]
mod duration_tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct PathGuard {
        previous: Option<OsString>,
    }

    impl Drop for PathGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }
    }

    fn install_fake_ffprobe(duration_seconds: &str) -> (tempfile::TempDir, PathGuard) {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let ffprobe = temp_dir.path().join("ffprobe");
        let payload = json_duration(duration_seconds);
        std::fs::write(&ffprobe, format!("#!/bin/sh\ncat <<'EOF'\n{payload}\nEOF\n")).unwrap();

        let mut permissions = std::fs::metadata(&ffprobe).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&ffprobe, permissions).unwrap();

        let previous = std::env::var_os("PATH");
        let mut paths = vec![temp_dir.path().to_path_buf()];
        if let Some(previous) = previous.as_ref() {
            paths.extend(std::env::split_paths(previous));
        }
        std::env::set_var("PATH", std::env::join_paths(paths).unwrap());

        (temp_dir, PathGuard { previous })
    }

    fn json_duration(duration_seconds: &str) -> String {
        format!(r#"{{"format":{{"duration":"{duration_seconds}"}}}}"#)
    }

    #[test]
    fn short_duration_movie_is_unknown() {
        let _env = env_lock();
        let (_temp_dir, _path) = install_fake_ffprobe("1800");
        let mut config = RoseyConfig::default();
        config.identification.movies_always_in_own_directory = false;

        let result =
            identify_file_with_config(&Utf8PathBuf::from("/media/short_clip.mkv"), &config);

        assert_eq!(result.item.kind, MediaKind::Unknown);
        assert!(result.reasons.iter().any(|reason| reason.contains("not a movie")));
    }

    #[test]
    fn nfo_movie_id_skips_duration_rejection() {
        let _env = env_lock();
        let (_ffprobe_dir, _path) = install_fake_ffprobe("1800");
        let temp_dir = tempfile::TempDir::new().unwrap();
        let media_file = temp_dir.path().join("movie.mkv");
        std::fs::write(&media_file, "").unwrap();
        std::fs::write(
            temp_dir.path().join("movie.nfo"),
            r#"<movie><uniqueid type="tmdb">603</uniqueid></movie>"#,
        )
        .unwrap();
        let media_file = Utf8PathBuf::from_path_buf(media_file).unwrap();

        let result = identify_file_with_config(&media_file, &RoseyConfig::default());

        assert_eq!(result.item.kind, MediaKind::Movie);
        assert_eq!(result.item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("603"));
    }
}
