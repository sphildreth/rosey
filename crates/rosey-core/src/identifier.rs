use camino::Utf8Path;
use std::collections::BTreeMap;

use crate::companions::discover_companion_files;
use crate::models::{MediaItem, MediaKind};
use crate::nfo::{find_nfo_for_file, parse_nfo};
use crate::patterns::{
    clean_title_with_year, extract_date, extract_episode_info, extract_part, extract_year,
};

pub fn identify_file(path: &Utf8Path) -> MediaItem {
    let filename = path.file_stem().unwrap_or(path.as_str());
    let folder_name = path.parent().and_then(|p| p.file_name()).unwrap_or("").to_string();

    let nfo_data = find_nfo_for_file(path).and_then(|p| parse_nfo(&p));
    let companions = discover_companion_files(path);

    let year = extract_year(filename).or_else(|| {
        if !folder_name.is_empty() {
            extract_year(&folder_name)
        } else {
            None
        }
    });

    let episode_info = extract_episode_info(filename, None).or_else(|| {
        if !folder_name.is_empty() {
            extract_episode_info(&folder_name, None)
        } else {
            None
        }
    });

    let date = extract_date(filename).map(|d| d.date);
    let part = extract_part(filename);

    let mut item = if let Some(ref nfo) = nfo_data {
        if nfo.season.is_some() || nfo.episode.is_some() {
            MediaItem {
                kind: MediaKind::Episode,
                source_path: path.to_path_buf(),
                title: nfo.title.clone(),
                year: nfo.year,
                season: nfo.season,
                episodes: nfo.episode.map(|e| vec![e]).unwrap_or_default(),
                part,
                date: date.clone(),
                sidecars: companions,
                nfo: build_nfo_map(nfo),
            }
        } else if nfo.tmdb_id.is_some() || nfo.imdb_id.is_some() {
            MediaItem {
                kind: MediaKind::Movie,
                source_path: path.to_path_buf(),
                title: nfo.title.clone(),
                year: nfo.year,
                season: None,
                episodes: Vec::new(),
                part,
                date: date.clone(),
                sidecars: companions,
                nfo: {
                    let mut map = BTreeMap::new();
                    if let Some(id) = &nfo.tmdb_id {
                        map.insert("tmdbid".to_string(), Some(id.clone()));
                    }
                    if let Some(id) = &nfo.imdb_id {
                        map.insert("imdbid".to_string(), Some(id.clone()));
                    }
                    map
                },
            }
        } else {
            MediaItem::unknown(path)
        }
    } else if let Some(ep) = episode_info {
        MediaItem {
            kind: MediaKind::Episode,
            source_path: path.to_path_buf(),
            title: build_title(filename, year),
            year,
            season: Some(ep.season),
            episodes: ep.episodes,
            part,
            date: date.clone(),
            sidecars: companions,
            nfo: BTreeMap::new(),
        }
    } else if date.is_some() {
        MediaItem {
            kind: MediaKind::Episode,
            source_path: path.to_path_buf(),
            title: build_title(filename, year),
            year,
            season: None,
            episodes: Vec::new(),
            part,
            date: date.clone(),
            sidecars: companions,
            nfo: BTreeMap::new(),
        }
    } else if year.is_some() || part.is_some() {
        MediaItem {
            kind: MediaKind::Movie,
            source_path: path.to_path_buf(),
            title: build_title(filename, year),
            year,
            season: None,
            episodes: Vec::new(),
            part,
            date: date.clone(),
            sidecars: companions,
            nfo: BTreeMap::new(),
        }
    } else {
        MediaItem::unknown(path)
    };

    if item.title.is_none() && !folder_name.is_empty() {
        let folder_year = extract_year(&folder_name);
        item.title = Some(clean_title_with_year(&folder_name, folder_year));
    }

    item
}

fn build_nfo_map(nfo: &crate::nfo::NfoData) -> BTreeMap<String, Option<String>> {
    let mut map = BTreeMap::new();
    if let Some(id) = &nfo.tmdb_id {
        map.insert("tmdbid".to_string(), Some(id.clone()));
    }
    if let Some(id) = &nfo.imdb_id {
        map.insert("imdbid".to_string(), Some(id.clone()));
    }
    if let Some(id) = &nfo.tvdb_id {
        map.insert("tvdbid".to_string(), Some(id.clone()));
    }
    if let Some(title) = &nfo.episode_title {
        map.insert("episode_title".to_string(), Some(title.clone()));
    }
    map
}

fn build_title(filename: &str, year: Option<u16>) -> Option<String> {
    let cleaned = clean_title_with_year(filename, year);
    if cleaned.is_empty() {
        Some(filename.to_string())
    } else {
        Some(cleaned)
    }
}
