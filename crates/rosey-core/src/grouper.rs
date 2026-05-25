use camino::Utf8Path;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::nfo::parse_nfo;
use crate::patterns::extract_episode_info;
use crate::patterns::extract_season_from_folder;

const GENERIC_ROOTS: &[&str] = &[
    "source",
    "sources",
    "tv",
    "movies",
    "movie",
    "video",
    "videos",
    "media",
    "downloads",
    "download",
    "incoming",
    "complete",
];

const PERMITTED_NESTED: &[&str] = &["subs", "subtitles", "extras"];

const SIDECAR_EXTENSIONS: &[&str] =
    &["srt", "ssa", "ass", "vtt", "sub", "idx", "sbv", "lrc", "smi", "stl", "nfo", "png"];

#[derive(Debug, Clone)]
pub struct MediaGroup {
    pub directory: String,
    pub kind: String,
    pub primary_videos: Vec<String>,
    pub companions: HashMap<String, Vec<String>>,
    pub directory_companions: Vec<String>,
    pub nfo_data: HashMap<String, Option<String>>,
    pub errors: Vec<String>,
}

impl MediaGroup {
    pub fn new(directory: String) -> Self {
        Self {
            directory,
            kind: "unknown".into(),
            primary_videos: Vec::new(),
            companions: HashMap::new(),
            directory_companions: Vec::new(),
            nfo_data: HashMap::new(),
            errors: Vec::new(),
        }
    }
}

pub fn get_media_directory(video_path: &str, root_path: &str) -> String {
    let video = Path::new(video_path);
    let root = Path::new(root_path);

    let mut current = video.parent();
    let mut candidates: Vec<&Path> = Vec::new();

    while let Some(dir) = current {
        if dir == root || dir == dir.parent().unwrap_or(dir) {
            break;
        }

        let folder_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if is_season_folder(folder_name) {
            candidates.push(dir.parent().unwrap_or(dir));
            break;
        }

        if !GENERIC_ROOTS.contains(&folder_name.to_lowercase().as_str()) {
            candidates.push(dir);
        }

        current = dir.parent();
    }

    if let Some(first) = candidates.first() {
        first.to_string_lossy().to_string()
    } else {
        video
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| root_path.to_string())
    }
}

fn is_season_folder(folder_name: &str) -> bool {
    extract_season_from_folder(folder_name).is_some()
        || PERMITTED_NESTED.contains(&folder_name.to_lowercase().as_str())
}

pub fn build_media_groups(
    video_files: &[String],
    root_path: &str,
    enforce_one_media: bool,
) -> Vec<MediaGroup> {
    let mut groups_dict: HashMap<String, MediaGroup> = HashMap::new();

    for video_path in video_files {
        let media_dir = get_media_directory(video_path, root_path);
        groups_dict
            .entry(media_dir.clone())
            .or_insert_with(|| MediaGroup::new(media_dir.clone()))
            .primary_videos
            .push(video_path.clone());
    }

    for group in groups_dict.values_mut() {
        discover_companions(group);
        parse_group_nfo(group);
    }

    for group in groups_dict.values_mut() {
        classify_group(group, enforce_one_media);
    }

    groups_dict.into_values().collect()
}

fn discover_companions(group: &mut MediaGroup) {
    let media_dir = Path::new(&group.directory);

    let mut all_files: Vec<PathBuf> = Vec::new();

    if media_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(media_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_file() {
                        all_files.push(entry.path());
                    } else if ft.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_lowercase();
                        if is_season_folder(&name) {
                            if let Ok(sub_entries) = std::fs::read_dir(entry.path()) {
                                for sub in sub_entries.filter_map(|e| e.ok()) {
                                    if sub.file_type().map(|t| t.is_file()).unwrap_or(false) {
                                        all_files.push(sub.path());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let primary_bases: Vec<String> = group
        .primary_videos
        .iter()
        .map(|v| Path::new(v).file_stem().unwrap_or_default().to_string_lossy().to_string())
        .collect();

    for file_path in &all_files {
        let file_str = file_path.to_string_lossy().to_string();
        if group.primary_videos.contains(&file_str) {
            continue;
        }

        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        if !SIDECAR_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }

        let base_name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();

        let mut matched = false;
        for primary_base in &primary_bases {
            if base_name == *primary_base || base_name.starts_with(&format!("{}.", primary_base)) {
                group.companions.entry(primary_base.clone()).or_default().push(file_str.clone());
                matched = true;
                break;
            }
        }

        if !matched {
            let common_names =
                ["movie", "tvshow", "show", "poster", "fanart", "banner", "landscape", "clearlogo"];
            if common_names.contains(&base_name.to_lowercase().as_str()) {
                group.directory_companions.push(file_str);
            }
        }
    }
}

fn parse_group_nfo(group: &mut MediaGroup) {
    let media_dir = Path::new(&group.directory);

    for nfo_name in ["movie.nfo", "tvshow.nfo", "show.nfo"] {
        let nfo_path = media_dir.join(nfo_name);
        if nfo_path.exists() {
            let utf8_path = Utf8Path::from_path(&nfo_path);
            if let Some(utf8_p) = utf8_path {
                if let Some(data) = parse_nfo(utf8_p) {
                    let mut map = HashMap::new();
                    map.insert("title".to_string(), data.title);
                    map.insert("year".to_string(), data.year.map(|y| y.to_string()));
                    if let Some(id) = data.tmdb_id {
                        map.insert("tmdbid".into(), Some(id));
                    }
                    if let Some(id) = data.imdb_id {
                        map.insert("imdbid".into(), Some(id));
                    }
                    if let Some(id) = data.tvdb_id {
                        map.insert("tvdbid".into(), Some(id));
                    }
                    if let Some(s) = data.season {
                        map.insert("season".into(), Some(s.to_string()));
                    }
                    if let Some(e) = data.episode {
                        map.insert("episode".into(), Some(e.to_string()));
                    }
                    group.nfo_data = map;
                }
            }
            break;
        }
    }
}

#[allow(clippy::collapsible_if)]
fn classify_group(group: &mut MediaGroup, enforce_one_media: bool) {
    let media_dir = Path::new(&group.directory);

    let has_season_folders = if media_dir.exists() {
        std::fs::read_dir(media_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .any(|e| {
                        let name = e.file_name().to_string_lossy().to_lowercase();
                        is_season_folder(&name)
                    })
            })
            .unwrap_or(false)
    } else {
        false
    };

    let has_episode_patterns = group.primary_videos.iter().any(|v| {
        let stem = Path::new(v).file_stem().and_then(|s| s.to_str()).unwrap_or("");
        extract_episode_info(stem, None).is_some()
    });

    let has_date_patterns = group.primary_videos.iter().any(|v| {
        let stem = Path::new(v).file_stem().and_then(|s| s.to_str()).unwrap_or("");
        has_date_pattern(stem)
    });

    let nfo_has_season = group.nfo_data.get("season").and_then(|s| s.as_ref()).is_some();

    if has_season_folders || has_episode_patterns || has_date_patterns || nfo_has_season {
        group.kind = "show".into();
    } else if group.primary_videos.len() == 1 {
        let has_movie_id = group.nfo_data.get("tmdbid").and_then(|s| s.as_ref()).is_some()
            || group.nfo_data.get("imdbid").and_then(|s| s.as_ref()).is_some();
        if has_movie_id && !nfo_has_season {
            group.kind = "movie".into();
        } else if nfo_has_season {
            group.kind = "show".into();
        } else {
            group.kind = "movie".into();
        }
    } else if group.primary_videos.len() > 1 {
        if enforce_one_media {
            group.kind = "unknown".into();
            group.errors.push(format!(
                "Mixed content: {} videos without clear classification",
                group.primary_videos.len()
            ));
        } else {
            let names: Vec<&str> = group
                .primary_videos
                .iter()
                .map(|v| Path::new(v).file_stem().and_then(|s| s.to_str()).unwrap_or(""))
                .collect();
            let prefix = find_common_prefix(&names);
            if prefix.len() > 3 {
                group.kind = "show".into();
            } else {
                group.kind = "unknown".into();
            }
        }
    }
}

fn has_date_pattern(filename: &str) -> bool {
    let bytes = filename.as_bytes();
    if bytes.len() < 10 {
        return false;
    }
    for i in 0..bytes.len() - 9 {
        if bytes[i].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
            && bytes[i + 4] == b'-'
            && bytes[i + 5].is_ascii_digit()
            && bytes[i + 6].is_ascii_digit()
            && bytes[i + 7] == b'-'
            && bytes[i + 8].is_ascii_digit()
            && bytes[i + 9].is_ascii_digit()
        {
            return true;
        }
    }
    false
}

fn find_common_prefix(names: &[&str]) -> String {
    if names.is_empty() {
        return String::new();
    }
    let mut prefix = names[0].to_string();
    for name in &names[1..] {
        let mut i = 0;
        while i < prefix.len() && i < name.len() && prefix.as_bytes()[i] == name.as_bytes()[i] {
            i += 1;
        }
        prefix.truncate(i);
        if prefix.is_empty() {
            break;
        }
    }
    prefix.trim().to_string()
}
