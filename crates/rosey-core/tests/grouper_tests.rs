#[cfg(test)]
mod grouper_tests {
    use rosey_core::{build_media_groups, get_media_directory};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn single_movie_directory_classified_as_movie() {
        let dir = TempDir::new().unwrap();

        let movie_dir = dir.path().join("Inception (2010)");
        fs::create_dir_all(&movie_dir).unwrap();
        fs::write(movie_dir.join("Inception (2010).mkv"), b"").unwrap();

        let video_files =
            vec![movie_dir.join("Inception (2010).mkv").to_string_lossy().to_string()];

        let groups = build_media_groups(&video_files, &dir.path().to_string_lossy(), false);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].kind, "movie");
        assert_eq!(groups[0].primary_videos.len(), 1);
    }

    #[test]
    fn show_with_season_folders_classified_as_show() {
        let dir = TempDir::new().unwrap();

        let show_dir = dir.path().join("The Wire");
        let season_dir = show_dir.join("Season 01");
        fs::create_dir_all(&season_dir).unwrap();
        fs::write(season_dir.join("The Wire - S01E01.mkv"), b"").unwrap();
        fs::write(season_dir.join("The Wire - S01E02.mkv"), b"").unwrap();

        let video_files = vec![
            season_dir.join("The Wire - S01E01.mkv").to_string_lossy().to_string(),
            season_dir.join("The Wire - S01E02.mkv").to_string_lossy().to_string(),
        ];

        let groups = build_media_groups(&video_files, &dir.path().to_string_lossy(), false);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].kind, "show");
        assert!(!groups[0].primary_videos.is_empty());
    }

    #[test]
    fn get_media_directory_returns_parent_of_season_folder() {
        let dir = TempDir::new().unwrap();
        let show_dir = dir.path().join("Breaking Bad");
        let season_dir = show_dir.join("Season 05");
        fs::create_dir_all(&season_dir).unwrap();
        let video_path = season_dir.join("Breaking Bad - S05E01.mkv");
        fs::write(&video_path, b"").unwrap();

        let result =
            get_media_directory(&video_path.to_string_lossy(), &dir.path().to_string_lossy());

        assert!(result.contains("Breaking Bad"));
        assert!(!result.contains("Season 05"));
    }

    #[test]
    fn generic_roots_are_skipped() {
        let dir = TempDir::new().unwrap();
        let movies_dir = dir.path().join("movies");
        let movie_sub_dir = movies_dir.join("Inception (2010)");
        fs::create_dir_all(&movie_sub_dir).unwrap();
        let video_path = movie_sub_dir.join("Inception (2010).mkv");
        fs::write(&video_path, b"").unwrap();

        let result =
            get_media_directory(&video_path.to_string_lossy(), &dir.path().to_string_lossy());

        assert!(result.contains("Inception (2010)"));
    }

    #[test]
    fn nfo_in_directory_sets_movie_type() {
        let dir = TempDir::new().unwrap();
        let movie_dir = dir.path().join("The Godfather");
        fs::create_dir_all(&movie_dir).unwrap();

        let nfo_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<movie>
  <title>The Godfather</title>
  <year>1972</year>
  <imdbid>tt0068646</imdbid>
</movie>"#;
        fs::write(movie_dir.join("movie.nfo"), nfo_content).unwrap();
        fs::write(movie_dir.join("The Godfather (1972).mkv"), b"").unwrap();

        let video_files =
            vec![movie_dir.join("The Godfather (1972).mkv").to_string_lossy().to_string()];

        let groups = build_media_groups(&video_files, &dir.path().to_string_lossy(), false);

        assert_eq!(groups.len(), 1);
        assert!(!groups[0].nfo_data.is_empty());
    }

    #[test]
    fn multiple_videos_with_common_prefix_classified_as_show() {
        let dir = TempDir::new().unwrap();
        let show_dir = dir.path().join("Example Show");
        fs::create_dir_all(&show_dir).unwrap();
        fs::write(show_dir.join("Example Show - S01E01.mkv"), b"").unwrap();
        fs::write(show_dir.join("Example Show - S01E02.mkv"), b"").unwrap();

        let video_files = vec![
            show_dir.join("Example Show - S01E01.mkv").to_string_lossy().to_string(),
            show_dir.join("Example Show - S01E02.mkv").to_string_lossy().to_string(),
        ];

        let groups = build_media_groups(&video_files, &dir.path().to_string_lossy(), false);

        assert_eq!(groups.len(), 1);
        assert_ne!(groups[0].kind, "unknown");
    }
}
