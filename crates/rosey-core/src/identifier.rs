use camino::Utf8Path;
use std::collections::BTreeMap;
use std::process::Command;
use std::time::Duration;

use crate::companions::discover_companion_files;
use crate::config::{IdentificationConfig, RoseyConfig};
use crate::models::{IdentificationResult, MediaItem, MediaKind};
use crate::nfo::{find_nfo_for_file, parse_nfo, NfoData};
use crate::patterns::{
    clean_title, clean_title_with_year, extract_date, extract_episode_info, extract_part,
    extract_season_from_folder, extract_title_before_episode, extract_year, DateMatch,
    EpisodeMatch,
};

const GENERIC_DIRS: &[&str] = &[
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

const MEDIA_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v"];

#[derive(Debug, Clone, Copy, Default)]
pub struct IdentifyOptions {
    pub skip_duration: bool,
}

pub fn identify_file(path: &Utf8Path) -> MediaItem {
    identify_file_with_config(path, &RoseyConfig::default()).item
}

pub fn identify_file_with_config(path: &Utf8Path, config: &RoseyConfig) -> IdentificationResult {
    identify_file_with_options(path, config, IdentifyOptions::default())
}

pub fn identify_file_fast(path: &Utf8Path, config: &RoseyConfig) -> IdentificationResult {
    identify_file_with_options(path, config, IdentifyOptions { skip_duration: true })
}

pub fn identify_file_with_options(
    path: &Utf8Path,
    config: &RoseyConfig,
    options: IdentifyOptions,
) -> IdentificationResult {
    let filename = path.file_stem().unwrap_or(path.as_str());
    let folder_name = path.parent().and_then(|p| p.file_name()).unwrap_or("");
    let parent_folder =
        path.parent().and_then(|p| p.parent()).and_then(|p| p.file_name()).unwrap_or("");
    let mut reasons = Vec::new();
    let mut errors = Vec::new();

    let nfo_path = find_nfo_for_file(path);
    let nfo_data = if let Some(nfo_path) = nfo_path {
        match parse_nfo(&nfo_path) {
            Some(data) => {
                reasons.push(format!(
                    "Found NFO file: {}",
                    nfo_path.file_name().unwrap_or(nfo_path.as_str())
                ));
                Some(data)
            }
            None => {
                errors.push(format!(
                    "Failed to parse NFO: {}",
                    nfo_path.file_name().unwrap_or(nfo_path.as_str())
                ));
                None
            }
        }
    } else {
        None
    };

    let known_season = extract_season_from_folder(folder_name)
        .or_else(|| extract_season_from_folder(parent_folder));
    let episode_info = extract_episode_info(filename, None)
        .or_else(|| known_season.and_then(|season| extract_episode_info(filename, Some(season))))
        .or_else(|| extract_episode_info(folder_name, None));
    let date_info = extract_date(filename);

    let item = if episode_info.is_some()
        || date_info.is_some()
        || nfo_data.as_ref().and_then(|nfo| nfo.season).is_some()
    {
        identify_episode(EpisodeContext {
            path,
            filename,
            folder_name,
            parent_folder,
            episode_info: episode_info.as_ref(),
            date_info: date_info.as_ref(),
            nfo_data: nfo_data.as_ref(),
            reasons: &mut reasons,
        })
    } else {
        identify_movie_with_constraints(
            path,
            filename,
            nfo_data.as_ref(),
            config,
            options,
            &mut reasons,
        )
    };

    IdentificationResult { item, reasons, errors }
}

fn identify_movie_with_constraints(
    path: &Utf8Path,
    filename: &str,
    nfo_data: Option<&NfoData>,
    config: &RoseyConfig,
    options: IdentifyOptions,
    reasons: &mut Vec<String>,
) -> MediaItem {
    let folder_name = path.parent().and_then(|p| p.file_name()).unwrap_or("");
    let has_nfo_movie_id = nfo_data
        .filter(|nfo| nfo.season.is_none())
        .map(|nfo| nfo.tmdb_id.is_some() || nfo.imdb_id.is_some())
        .unwrap_or(false);

    if has_nfo_movie_id {
        if config.identification.movies_always_in_own_directory {
            if let Some(item) = directory_constraint_unknown(path, filename, reasons) {
                return item;
            }
            let item = identify_movie(path, filename, nfo_data, reasons);
            reasons.push("NFO with TMDB/IMDB ID and directory constraints satisfied".to_string());
            return item;
        }
        return identify_movie(path, filename, nfo_data, reasons);
    }

    let duration = if should_check_duration(&config.identification, options) {
        video_duration_minutes(path)
    } else {
        None
    };
    let has_movie_hint = extract_year(filename).is_some()
        || extract_year(folder_name).is_some()
        || extract_part(filename).is_some()
        || nfo_data.and_then(|nfo| nfo.year).is_some();
    let has_nfo_title = nfo_data.and_then(|nfo| nfo.title.as_ref()).is_some();

    if has_movie_hint {
        let success_reason = config
            .identification
            .movies_always_in_own_directory
            .then_some("Directory constraints and duration satisfied");
        return identify_movie_after_checks(
            path,
            filename,
            nfo_data,
            config,
            duration,
            true,
            "Short duration - classified as unknown",
            success_reason,
            reasons,
        );
    }

    if has_nfo_title {
        let success_reason = config
            .identification
            .movies_always_in_own_directory
            .then_some("Directory constraints and duration satisfied");
        return identify_movie_after_checks(
            path,
            filename,
            nfo_data,
            config,
            duration,
            false,
            "Short duration despite NFO title",
            success_reason,
            reasons,
        );
    }

    let success_reason = if config.identification.movies_always_in_own_directory {
        "No episode pattern - defaulting to movie and directory constraints satisfied"
    } else {
        "No episode pattern - defaulting to movie"
    };
    identify_movie_after_checks(
        path,
        filename,
        nfo_data,
        config,
        duration,
        true,
        "Short duration - classified as unknown",
        Some(success_reason),
        reasons,
    )
}

#[allow(clippy::too_many_arguments)]
fn identify_movie_after_checks(
    path: &Utf8Path,
    filename: &str,
    nfo_data: Option<&NfoData>,
    config: &RoseyConfig,
    duration: Option<f64>,
    short_duration_is_unknown: bool,
    short_duration_reason: &str,
    success_reason: Option<&str>,
    reasons: &mut Vec<String>,
) -> MediaItem {
    if config.identification.movies_always_in_own_directory {
        if let Some(item) = directory_constraint_unknown(path, filename, reasons) {
            return item;
        }
    }

    if let Some(duration) = duration {
        let minimum = config.identification.minimum_movie_duration_minutes as f64;
        if duration < minimum {
            reasons.push(format!("Duration {duration:.1}min < {minimum:.0}min - not a movie"));
            if short_duration_is_unknown {
                reasons.push(short_duration_reason.to_string());
                return unknown_with_title(path, filename);
            }

            let item = identify_movie(path, filename, nfo_data, reasons);
            reasons.push(short_duration_reason.to_string());
            return item;
        }
        reasons.push(format!("Duration {duration:.1}min meets minimum"));
    }

    let item = identify_movie(path, filename, nfo_data, reasons);
    if let Some(success_reason) = success_reason {
        reasons.push(success_reason.to_string());
    }
    item
}

fn directory_constraint_unknown(
    path: &Utf8Path,
    filename: &str,
    reasons: &mut Vec<String>,
) -> Option<MediaItem> {
    if is_in_show_folder(path) {
        reasons.push(
            "File in show folder - cannot be movie when movies_always_in_own_directory is enabled"
                .to_string(),
        );
        return Some(unknown_with_title(path, filename));
    }
    if !is_only_media_file_in_directory(path) {
        reasons.push(
            "Directory contains multiple media files - cannot be movie when movies_always_in_own_directory is enabled"
                .to_string(),
        );
        return Some(unknown_with_title(path, filename));
    }
    None
}

struct EpisodeContext<'a> {
    path: &'a Utf8Path,
    filename: &'a str,
    folder_name: &'a str,
    parent_folder: &'a str,
    episode_info: Option<&'a EpisodeMatch>,
    date_info: Option<&'a DateMatch>,
    nfo_data: Option<&'a NfoData>,
    reasons: &'a mut Vec<String>,
}

fn identify_episode(ctx: EpisodeContext<'_>) -> MediaItem {
    let EpisodeContext {
        path,
        filename,
        folder_name,
        parent_folder,
        episode_info,
        date_info,
        nfo_data,
        reasons,
    } = ctx;
    let title = if let Some(title) = nfo_data.and_then(|nfo| nfo.title.clone()) {
        reasons.push("Show title from NFO".to_string());
        Some(title)
    } else {
        Some(derive_show_title(filename, folder_name, parent_folder, reasons))
    };

    let (season, episodes, episode_title_from_filename) = if let Some(ep) = episode_info {
        reasons.push(format!("Parsed episode: S{:02}E{:02}", ep.season, ep.episodes[0]));
        if let Some(title) = &ep.title {
            reasons.push(format!("Episode title from filename: {title}"));
        }
        (Some(ep.season), ep.episodes.clone(), ep.title.clone())
    } else if let Some(nfo) = nfo_data {
        reasons.push("Season/episode from NFO".to_string());
        (nfo.season, nfo.episode.map(|episode| vec![episode]).unwrap_or_default(), None)
    } else {
        let season = extract_season_from_folder(folder_name);
        if let Some(season) = season {
            reasons.push(format!("Season {season} from folder"));
        }
        (season, Vec::new(), None)
    };

    let part = extract_part(filename);
    if let Some(part) = part {
        reasons.push(format!("Multipart episode: Part {part}"));
    }

    let year = nfo_data
        .and_then(|nfo| nfo.year)
        .or_else(|| extract_year(parent_folder))
        .or_else(|| extract_year(folder_name));
    if let Some(year) = year {
        reasons.push(format!("Year {year} parsed from folder"));
    }

    let mut nfo = build_nfo_map(nfo_data);
    if let Some(title) = episode_title_from_filename {
        nfo.entry("episode_title".to_string()).or_insert(Some(title));
    }

    MediaItem {
        kind: MediaKind::Episode,
        source_path: path.to_path_buf(),
        title,
        year,
        season,
        episodes,
        part,
        date: date_info.map(|date| date.date.clone()),
        sidecars: discover_companion_files(path),
        nfo,
    }
}

fn identify_movie(
    path: &Utf8Path,
    filename: &str,
    nfo_data: Option<&NfoData>,
    reasons: &mut Vec<String>,
) -> MediaItem {
    let (title, year) = if let Some(nfo) = nfo_data.filter(|nfo| nfo.title.is_some()) {
        reasons.push("Movie title and year from NFO".to_string());
        (nfo.title.clone(), nfo.year)
    } else {
        let year = extract_year(filename);
        let title = Some(clean_title_with_year(filename, year));
        reasons.push("Movie title from filename".to_string());
        if let Some(year) = year {
            reasons.push(format!("Year {year} parsed from filename"));
        }
        (title, year)
    };

    let part = extract_part(filename);
    if let Some(part) = part {
        reasons.push(format!("Multipart movie: Part {part}"));
    }

    let sidecars = discover_companion_files(path);
    if !sidecars.is_empty() {
        reasons.push(format!("Found {} companion files", sidecars.len()));
    }

    MediaItem {
        kind: MediaKind::Movie,
        source_path: path.to_path_buf(),
        title,
        year,
        season: None,
        episodes: Vec::new(),
        part,
        date: None,
        sidecars,
        nfo: build_nfo_map(nfo_data),
    }
}

fn derive_show_title(
    filename: &str,
    folder_name: &str,
    parent_folder: &str,
    reasons: &mut Vec<String>,
) -> String {
    let parent_clean = clean_title(parent_folder);
    let folder_clean = clean_title(folder_name);
    let file_title_part = extract_title_before_episode(filename);
    let file_clean = clean_title(&file_title_part);
    let is_season_dir = extract_season_from_folder(folder_name).is_some();

    if is_season_dir && !parent_clean.is_empty() && !is_generic_dir(parent_folder) {
        reasons.push("Show title from folder structure".to_string());
        parent_clean
    } else if !folder_clean.is_empty() && !is_generic_dir(folder_name) {
        reasons.push("Show title from folder structure".to_string());
        folder_clean
    } else if !parent_clean.is_empty() && !is_generic_dir(parent_folder) {
        reasons.push("Show title from folder structure".to_string());
        parent_clean
    } else {
        reasons.push("Show title from filename".to_string());
        file_clean
    }
}

fn is_generic_dir(name: &str) -> bool {
    GENERIC_DIRS.iter().any(|generic| name.eq_ignore_ascii_case(generic))
}

fn build_nfo_map(nfo: Option<&NfoData>) -> BTreeMap<String, Option<String>> {
    let mut map = BTreeMap::new();
    let Some(nfo) = nfo else {
        return map;
    };

    if let Some(id) = &nfo.tmdb_id {
        map.insert("tmdbid".to_string(), Some(id.clone()));
    }
    if let Some(id) = &nfo.imdb_id {
        map.insert("imdbid".to_string(), Some(id.clone()));
    }
    if let Some(id) = &nfo.tvdb_id {
        map.insert("tvdbid".to_string(), Some(id.clone()));
    }
    if let Some(title) = &nfo.title {
        map.insert("title".to_string(), Some(title.clone()));
    }
    if let Some(title) = &nfo.episode_title {
        map.insert("episode_title".to_string(), Some(title.clone()));
    }
    map
}

fn unknown_with_title(path: &Utf8Path, filename: &str) -> MediaItem {
    let mut item = MediaItem::unknown(path);
    let title = clean_title(filename);
    if !title.is_empty() {
        item.title = Some(title);
    }
    item
}

fn should_check_duration(config: &IdentificationConfig, options: IdentifyOptions) -> bool {
    !options.skip_duration && config.minimum_movie_duration_minutes > 0
}

fn video_duration_minutes(path: &Utf8Path) -> Option<f64> {
    let output = Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_format", path.as_str()])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let duration = value.get("format")?.get("duration")?.as_str()?.parse::<f64>().ok()?;
    Some(duration / 60.0)
}

fn is_in_show_folder(path: &Utf8Path) -> bool {
    let Some(directory) = path.parent() else {
        return false;
    };

    if directory.file_name().and_then(extract_season_from_folder).is_some() {
        return true;
    }

    let mut media_count = 0;
    if let Ok(entries) = std::fs::read_dir(directory.as_std_path()) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let Ok(path) = camino::Utf8PathBuf::from_path_buf(entry_path) else {
                continue;
            };

            if path.is_dir() && path.file_name().and_then(extract_season_from_folder).is_some() {
                return true;
            }

            if path.is_file() && is_media_path(&path) {
                media_count += 1;
                if media_count >= 2 {
                    return true;
                }
            }
        }
    }

    false
}

fn is_only_media_file_in_directory(path: &Utf8Path) -> bool {
    let Some(directory) = path.parent() else {
        return true;
    };

    let Ok(entries) = std::fs::read_dir(directory.as_std_path()) else {
        return true;
    };

    for entry in entries.flatten() {
        let Ok(candidate) = camino::Utf8PathBuf::from_path_buf(entry.path()) else {
            continue;
        };
        if candidate != path && candidate.is_file() && is_media_path(&candidate) {
            return false;
        }
    }

    true
}

fn is_media_path(path: &Utf8Path) -> bool {
    path.extension()
        .map(|ext| MEDIA_EXTENSIONS.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)))
        .unwrap_or(false)
}

#[allow(dead_code)]
fn _duration_timeout() -> Duration {
    Duration::from_secs(10)
}
