#[cfg(test)]
mod proptest_parser_tests {
    use proptest::prelude::*;
    use rosey_core::*;

    proptest! {
        #[test]
        fn extract_year_never_panics(fname in ".*") {
            let _ = extract_year(&fname);
        }

        #[test]
        fn extract_episode_never_panics(fname in ".*") {
            let _ = extract_episode_info(&fname, None);
        }

        #[test]
        fn extract_part_never_panics(fname in ".*") {
            let _ = extract_part(&fname);
        }

        #[test]
        fn extract_date_never_panics(fname in ".*") {
            let _ = extract_date(&fname);
        }

        #[test]
        fn extract_season_never_panics(fname in ".*") {
            let _ = extract_season_from_folder(&fname);
        }

        #[test]
        fn extract_tmdb_id_never_panics(path in ".*") {
            let _ = extract_tmdb_id_from_path(&path);
        }

        #[test]
        fn extract_title_before_episode_never_panics(fname in ".*") {
            let _ = extract_title_before_episode(&fname);
        }

        #[test]
        fn clean_title_never_panics(fname in ".*") {
            let _ = clean_title(&fname);
        }

        #[test]
        fn clean_title_with_year_never_panics(fname in ".*", year in 1900u16..2050u16) {
            let _ = clean_title_with_year(&fname, Some(year));
        }

        #[test]
        fn identifier_never_panics(path in "[a-zA-Z0-9._\\- /]+\\.mkv") {
            let fname = std::path::Path::new(&path);
            let utf8 = camino::Utf8Path::from_path(fname);
            if let Some(utf) = utf8 {
                let _ = identify_file(utf);
            }
        }

        #[test]
        fn score_never_panics(
            title in proptest::option::of("[a-zA-Z ]+"),
            year in proptest::option::of(1900u16..2040u16),
            season in proptest::option::of(1u16..30u16),
            episodes in proptest::collection::vec(1u16..9999u16, 0..5)
        ) {
            use std::collections::BTreeMap;
            let item = MediaItem {
                kind: if season.is_some() && !episodes.is_empty() { MediaKind::Episode }
                      else if year.is_some() { MediaKind::Movie }
                      else { MediaKind::Unknown },
                source_path: camino::Utf8PathBuf::from("/test/movie.mkv"),
                title,
                year,
                season,
                episodes,
                part: None,
                date: None,
                sidecars: Vec::new(),
                nfo: BTreeMap::new(),
            };
            let _ = score_identification(&item);
        }

        #[test]
        fn sanitize_name_never_panics(input in ".*") {
            let _ = sanitize_name(&input);
        }

        #[test]
        fn title_case_never_panics(input in ".*") {
            let _ = title_case(&input);
        }

        #[test]
        fn plan_path_never_panics(
            title in proptest::option::of("[a-zA-Z ]+"),
            year in proptest::option::of(1900u16..2040u16),
        ) {
            use std::collections::BTreeMap;
            let item = MediaItem {
                kind: MediaKind::Movie,
                source_path: camino::Utf8PathBuf::from("/test/movie.mkv"),
                title,
                year,
                season: None,
                episodes: Vec::new(),
                part: None,
                date: None,
                sidecars: Vec::new(),
                nfo: BTreeMap::new(),
            };
            let _ = plan_path(&item, "/movies", "/tv");
        }
    }
}
