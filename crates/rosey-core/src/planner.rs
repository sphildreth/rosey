use crate::MediaItem;
use camino::Utf8PathBuf;
use once_cell::sync::Lazy;
use regex::Regex;

const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

static WINDOWS_RESERVED: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .iter()
    .copied()
    .collect()
});

static LOWERCASE_WORDS: Lazy<std::collections::HashSet<&'static str>> = Lazy::new(|| {
    [
        "a", "an", "and", "as", "at", "but", "by", "for", "if", "in", "nor", "of", "on", "or",
        "so", "the", "to", "up", "yet",
    ]
    .iter()
    .copied()
    .collect()
});

/// Title-case a string, leaving common articles and prepositions lowercased
/// (except when they are the first word).
pub fn title_case(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return text.to_string();
    }

    let mut result = Vec::with_capacity(words.len());
    result.push(capitalize_first(words[0]));

    for word in &words[1..] {
        if LOWERCASE_WORDS.contains(word.to_lowercase().as_str()) {
            result.push(word.to_lowercase());
        } else {
            result.push(capitalize_first(word));
        }
    }

    result.join(" ")
}

fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        None => word.to_string(),
    }
}

/// Sanitize a filename or folder name for cross-platform compatibility.
pub fn sanitize_name(name: &str) -> String {
    let mut out = name.to_string();

    for c in INVALID_CHARS {
        out = out.replace(*c, "");
    }

    static MULTI_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
    out = MULTI_SPACE.replace_all(&out, " ").to_string();

    out = out.trim_matches(|c: char| c == ' ' || c == '.').to_string();

    let base_name = out.split('.').next().unwrap_or("").to_uppercase();
    if WINDOWS_RESERVED.contains(base_name.as_str()) {
        out = format!("{}_media", out);
    }

    if out.is_empty() {
        out = "unknown".to_string();
    }

    out
}

/// Plans Jellyfin-compatible destination paths for media items.
pub struct Planner {
    pub movies_root: Utf8PathBuf,
    pub tv_root: Utf8PathBuf,
}

impl Planner {
    pub fn plan_destination(&self, item: &MediaItem) -> Utf8PathBuf {
        match item.kind {
            crate::MediaKind::Movie => self.plan_movie(item),
            crate::MediaKind::Episode => self.plan_episode(item),
            _ => item.source_path.clone(),
        }
    }

    fn plan_movie(&self, item: &MediaItem) -> Utf8PathBuf {
        if self.movies_root.as_str().is_empty() {
            return item.source_path.clone();
        }

        let title = sanitize_name(item.title.as_deref().unwrap_or("Unknown"));
        let mut folder_name = if let Some(year) = item.year {
            format!("{} ({})", title, year)
        } else {
            title.clone()
        };

        if let Some(tmdbid) = item.nfo.get("tmdbid").and_then(|v| v.as_ref()) {
            folder_name = format!("{} [tmdbid-{}]", folder_name, tmdbid);
        }

        let ext = item.source_path.extension().unwrap_or("");
        let filename = if let Some(part) = item.part {
            sanitize_name(&format!("{} Part {}.{}", folder_name, part, ext))
        } else {
            sanitize_name(&format!("{}.{}", folder_name, ext))
        };

        self.movies_root.join(&folder_name).join(filename)
    }

    fn plan_episode(&self, item: &MediaItem) -> Utf8PathBuf {
        if self.tv_root.as_str().is_empty() {
            return item.source_path.clone();
        }

        let title = title_case(item.title.as_deref().unwrap_or("Unknown Show"));
        let title_sanitized = sanitize_name(&title);
        let mut show_folder = if let Some(year) = item.year {
            format!("{} ({})", title_sanitized, year)
        } else {
            title_sanitized.clone()
        };

        if let Some(tmdbid) = item.nfo.get("tmdbid").and_then(|v| v.as_ref()) {
            show_folder = format!("{} [tmdbid-{}]", show_folder, tmdbid);
        }

        let season_num = item.season.unwrap_or(0);
        let season_folder = format!("Season {:02}", season_num);

        let ext = item.source_path.extension().unwrap_or("");
        let filename = if let Some(ref date) = item.date {
            sanitize_name(&format!("{} - {}.{}", title_sanitized, date, ext))
        } else if !item.episodes.is_empty() {
            let ep_str = if item.episodes.len() == 1 {
                format!("S{:02}E{:02}", season_num, item.episodes[0])
            } else {
                format!(
                    "S{:02}E{:02}-E{:02}",
                    season_num,
                    item.episodes[0],
                    item.episodes[item.episodes.len() - 1]
                )
            };

            if let Some(part) = item.part {
                sanitize_name(&format!("{} - {} Part {}.{}", title_sanitized, ep_str, part, ext))
            } else if let Some(ep_title) = item.nfo.get("episode_title").and_then(|v| v.as_ref()) {
                let ep_title_clean = sanitize_name(ep_title);
                sanitize_name(&format!(
                    "{} - {} - {}.{}",
                    title_sanitized, ep_str, ep_title_clean, ext
                ))
            } else {
                sanitize_name(&format!("{} - {}.{}", title_sanitized, ep_str, ext))
            }
        } else {
            sanitize_name(&format!("{}.{}", title_sanitized, ext))
        };

        self.tv_root.join(show_folder).join(season_folder).join(filename)
    }
}

/// Convenience function to plan a destination path.
pub fn plan_path(
    item: &MediaItem,
    movies_root: impl Into<Utf8PathBuf>,
    tv_root: impl Into<Utf8PathBuf>,
) -> Utf8PathBuf {
    let planner = Planner { movies_root: movies_root.into(), tv_root: tv_root.into() };
    planner.plan_destination(item)
}
