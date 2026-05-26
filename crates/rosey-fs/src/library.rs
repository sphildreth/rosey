use camino::Utf8Path;
use rosey_core::{
    extract_tmdb_id_from_path, find_nfo_for_file, identify_file_with_options, parse_nfo,
    IdentifyOptions, LibraryMovie, MediaKind, RoseyConfig,
};

use crate::{scan, ScanOptions};

#[derive(Debug, Clone)]
pub struct MovieLibraryIndexOptions {
    pub follow_symlinks: bool,
    pub max_workers: usize,
}

impl Default for MovieLibraryIndexOptions {
    fn default() -> Self {
        Self { follow_symlinks: false, max_workers: 8 }
    }
}

pub fn index_movie_library(
    root: &Utf8Path,
    options: MovieLibraryIndexOptions,
) -> Vec<LibraryMovie> {
    let mut config = RoseyConfig::default();
    config.identification.movies_always_in_own_directory = false;

    scan(
        root,
        ScanOptions {
            follow_symlinks: options.follow_symlinks,
            max_depth: None,
            max_workers: options.max_workers,
        },
    )
    .into_iter()
    .filter(|result| result.is_video && result.error.is_none())
    .filter_map(|result| {
        let ident = identify_file_with_options(
            &result.path,
            &config,
            IdentifyOptions { skip_duration: true },
        );

        if ident.item.kind == MediaKind::Episode {
            return None;
        }

        let nfo_data = find_nfo_for_file(&result.path).and_then(|nfo_path| parse_nfo(&nfo_path));
        let tmdb_id = ident
            .item
            .nfo
            .get("tmdbid")
            .cloned()
            .flatten()
            .or_else(|| nfo_data.as_ref().and_then(|nfo| nfo.tmdb_id.clone()))
            .or_else(|| extract_tmdb_id_from_path(result.path.as_str()));
        let imdb_id = ident
            .item
            .nfo
            .get("imdbid")
            .cloned()
            .flatten()
            .or_else(|| nfo_data.and_then(|nfo| nfo.imdb_id));

        Some(LibraryMovie {
            source_path: result.path,
            tmdb_id,
            imdb_id,
            title: ident.item.title,
            year: ident.item.year,
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn indexes_movie_library_with_path_tmdb_id_and_nfo_imdb_id() {
        let tmp = tempfile::tempdir().unwrap();
        let movie_dir = tmp.path().join("The Godfather (1972) [tmdbid-238]");
        fs::create_dir_all(&movie_dir).unwrap();
        fs::write(movie_dir.join("The Godfather (1972).mkv"), "video").unwrap();
        fs::write(
            movie_dir.join("movie.nfo"),
            r#"<movie><title>The Godfather</title><year>1972</year><imdbid>tt0068646</imdbid></movie>"#,
        )
        .unwrap();

        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let movies = index_movie_library(root, MovieLibraryIndexOptions::default());

        assert_eq!(movies.len(), 1);
        assert_eq!(movies[0].tmdb_id.as_deref(), Some("238"));
        assert_eq!(movies[0].imdb_id.as_deref(), Some("tt0068646"));
        assert_eq!(movies[0].title.as_deref(), Some("The Godfather"));
        assert_eq!(movies[0].year, Some(1972));
    }
}
