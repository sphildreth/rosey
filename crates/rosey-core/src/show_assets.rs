use camino::{Utf8Path, Utf8PathBuf};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs;

use crate::models::{ShowAsset, ShowAssetKind, TvShow};
use crate::patterns::extract_season_from_folder;

static SHOW_IMAGE_NAMES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "poster",
        "folder",
        "cover",
        "default",
        "show",
        "backdrop",
        "fanart",
        "background",
        "art",
        "banner",
        "logo",
        "clearlogo",
        "landscape",
        "thumb",
    ]
    .iter()
    .copied()
    .collect()
});

static SHOW_IMAGE_EXTS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["jpg", "jpeg", "png", "webp", "gif", "tiff", "bmp"].iter().copied().collect());

static SEASON_IMAGE_NAMES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "poster",
        "folder",
        "cover",
        "default",
        "backdrop",
        "fanart",
        "background",
        "art",
        "banner",
        "logo",
        "clearlogo",
        "landscape",
        "thumb",
    ]
    .iter()
    .copied()
    .collect()
});

static EXTRAS_FOLDER_NAMES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "behind the scenes",
        "deleted scenes",
        "interviews",
        "scenes",
        "samples",
        "shorts",
        "featurettes",
        "clips",
        "other",
        "extras",
        "trailers",
        "theme-music",
        "backdrops",
        "extrafanart",
    ]
    .iter()
    .copied()
    .collect()
});

static THEME_MEDIA_BASENAMES: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["theme"].iter().copied().collect());

static VIDEO_EXTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "ts"]
        .iter()
        .copied()
        .collect()
});

static NFO_NAMES: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["tvshow.nfo", "show.nfo"].iter().copied().collect());

static SUBTITLE_EXTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["srt", "ssa", "ass", "vtt", "sub", "idx", "sbv", "lrc", "smi", "stl"].iter().copied().collect()
});

static AUDIO_EXTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["mp3", "flac", "wav", "aac", "ogg", "m4a", "wma", "dts"].iter().copied().collect()
});

/// Discover show-level and season-level assets for a TV show.
///
/// Returns assets with `destination` paths relative to the show root.
/// Callers should use `TvShow::resolve_asset_destinations` to produce
/// absolute destination paths including the `tv_root`.
pub fn discover_show_assets(show: &TvShow) -> Vec<ShowAsset> {
    let mut assets = Vec::new();
    let show_dir = &show.source_root;

    if !show_dir.is_dir() {
        return assets;
    }

    if let Ok(entries) = fs::read_dir(show_dir.as_std_path()) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = match Utf8PathBuf::from_path_buf(entry.path()) {
                Ok(p) => p,
                Err(_) => continue,
            };

            if path.is_file() {
                if let Some(asset) = classify_show_file(&path, show) {
                    assets.push(asset);
                }
            } else if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    let dir_lower = dir_name.to_lowercase();

                    if dir_lower == "theme-music" {
                        assets.extend(discover_folder_assets(
                            &path,
                            show,
                            ShowAssetKind::ThemeMusic,
                            "theme-music",
                        ));
                    } else if dir_lower == "backdrops" || dir_lower == "extrafanart" {
                        assets.extend(discover_folder_assets(
                            &path,
                            show,
                            ShowAssetKind::Backdrop,
                            "backdrops",
                        ));
                    } else if EXTRAS_FOLDER_NAMES.contains(dir_lower.as_str()) {
                        assets.extend(discover_folder_assets(
                            &path,
                            show,
                            ShowAssetKind::ExtrasFolder,
                            &dir_lower,
                        ));
                    } else if let Some(season_num) = extract_season_from_folder(dir_name) {
                        assets.extend(discover_season_assets(&path, season_num, show));
                    }
                }
            }
        }
    }

    assets
}

fn classify_show_file(path: &Utf8Path, show: &TvShow) -> Option<ShowAsset> {
    let base_name = path.file_stem().unwrap_or("").to_lowercase();
    let ext = path.extension().unwrap_or("").to_lowercase();

    if NFO_NAMES.contains(path.file_name().unwrap_or("").to_lowercase().as_str()) {
        let file_name = path.file_name().unwrap_or("unknown");
        return Some(ShowAsset {
            source_path: path.to_path_buf(),
            asset_kind: ShowAssetKind::Nfo,
            destination: show.relative_destination(file_name),
        });
    }

    if THEME_MEDIA_BASENAMES.contains(base_name.as_str()) && is_media_ext(&ext) {
        let file_name = path.file_name().unwrap_or("unknown");
        return Some(ShowAsset {
            source_path: path.to_path_buf(),
            asset_kind: ShowAssetKind::ThemeMusic,
            destination: show.relative_destination(file_name),
        });
    }

    if SHOW_IMAGE_EXTS.contains(ext.as_str()) {
        for &image_name in SHOW_IMAGE_NAMES.iter() {
            if base_name == image_name || base_name.starts_with(&format!("{}.", image_name)) {
                let kind = classify_image_name(image_name);
                let dest_file = path.file_name().unwrap_or("unknown");
                return Some(ShowAsset {
                    source_path: path.to_path_buf(),
                    asset_kind: kind,
                    destination: show.relative_destination(dest_file),
                });
            }
        }
    }

    None
}

fn classify_image_name(name: &str) -> ShowAssetKind {
    match name {
        "poster" | "folder" | "cover" | "default" | "show" => ShowAssetKind::Poster,
        "backdrop" | "fanart" | "background" | "art" => ShowAssetKind::Backdrop,
        "banner" => ShowAssetKind::Banner,
        "logo" | "clearlogo" => ShowAssetKind::Logo,
        "landscape" | "thumb" => ShowAssetKind::Landscape,
        _ => ShowAssetKind::Other,
    }
}

fn classify_season_image(name: &str) -> ShowAssetKind {
    match name {
        "poster" | "folder" | "cover" | "default" => ShowAssetKind::SeasonPoster,
        "backdrop" | "fanart" | "background" | "art" => ShowAssetKind::SeasonBackdrop,
        "banner" => ShowAssetKind::SeasonBanner,
        "logo" | "clearlogo" => ShowAssetKind::Logo,
        "landscape" | "thumb" => ShowAssetKind::SeasonThumb,
        _ => ShowAssetKind::Other,
    }
}

fn discover_folder_assets(
    folder_path: &Utf8Path,
    show: &TvShow,
    kind: ShowAssetKind,
    subfolder_name: &str,
) -> Vec<ShowAsset> {
    let mut assets = Vec::new();
    let Ok(entries) = fs::read_dir(folder_path.as_std_path()) else {
        return assets;
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = match Utf8PathBuf::from_path_buf(entry.path()) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if path.is_file() {
            let ext = path.extension().unwrap_or("").to_lowercase();
            if is_media_ext(&ext) || SHOW_IMAGE_EXTS.contains(ext.as_str()) {
                let file_name = path.file_name().unwrap_or("unknown");
                assets.push(ShowAsset {
                    source_path: path.to_path_buf(),
                    asset_kind: kind,
                    destination: show
                        .relative_destination(&format!("{}/{}", subfolder_name, file_name)),
                });
            }
        }
    }

    assets
}

fn discover_season_assets(
    season_dir: &Utf8Path,
    season_number: u16,
    show: &TvShow,
) -> Vec<ShowAsset> {
    let mut assets = Vec::new();
    let Ok(entries) = fs::read_dir(season_dir.as_std_path()) else {
        return assets;
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = match Utf8PathBuf::from_path_buf(entry.path()) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if path.is_file() {
            if let Some(asset) = classify_season_file(&path, season_number, show) {
                assets.push(asset);
            }
        } else if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                let dir_lower = dir_name.to_lowercase();
                if dir_lower == "theme-music" {
                    assets.extend(discover_folder_assets(
                        &path,
                        show,
                        ShowAssetKind::ThemeMusic,
                        &format!("Season {:02}/theme-music", season_number),
                    ));
                } else if dir_lower == "backdrops" || dir_lower == "extrafanart" {
                    assets.extend(discover_folder_assets(
                        &path,
                        show,
                        ShowAssetKind::SeasonBackdrop,
                        &format!("Season {:02}/backdrops", season_number),
                    ));
                } else if EXTRAS_FOLDER_NAMES.contains(dir_lower.as_str()) {
                    assets.extend(discover_folder_assets(
                        &path,
                        show,
                        ShowAssetKind::ExtrasFolder,
                        &format!("Season {:02}/{}", season_number, dir_lower),
                    ));
                }
            }
        }
    }

    assets
}

fn classify_season_file(path: &Utf8Path, season_number: u16, show: &TvShow) -> Option<ShowAsset> {
    let base_name = path.file_stem().unwrap_or("").to_lowercase();
    let ext = path.extension().unwrap_or("").to_lowercase();

    if !SHOW_IMAGE_EXTS.contains(ext.as_str())
        && !SUBTITLE_EXTS.contains(ext.as_str())
        && !is_media_ext(&ext)
    {
        return None;
    }

    let season_folder = format!("Season {:02}", season_number);
    let season_prefix = format!("season {:02}", season_number);

    if SHOW_IMAGE_EXTS.contains(ext.as_str()) {
        for &image_name in SEASON_IMAGE_NAMES.iter() {
            if base_name == image_name
                || base_name == format!("{}_s{:02}", image_name, season_number)
                || base_name == format!("{} s{:02}", image_name, season_number)
                || base_name.starts_with(&format!("{}.", image_name))
                || base_name == format!("{}-s{:02}", image_name, season_number)
            {
                let kind = classify_season_image(image_name);
                let file_name = path.file_name().unwrap_or("unknown");
                return Some(ShowAsset {
                    source_path: path.to_path_buf(),
                    asset_kind: kind,
                    destination: show
                        .relative_destination(&format!("{}/{}", season_folder, file_name)),
                });
            }
        }

        if base_name.starts_with(&season_prefix) || base_name == format!("s{:02}", season_number) {
            let file_name = path.file_name().unwrap_or("unknown");
            return Some(ShowAsset {
                source_path: path.to_path_buf(),
                asset_kind: ShowAssetKind::SeasonPoster,
                destination: show.relative_destination(&format!("{}/{}", season_folder, file_name)),
            });
        }
    }

    if SUBTITLE_EXTS.contains(ext.as_str()) {
        let file_name = path.file_name().unwrap_or("unknown");
        return Some(ShowAsset {
            source_path: path.to_path_buf(),
            asset_kind: ShowAssetKind::EpisodeSubtitle,
            destination: show.relative_destination(&format!("{}/{}", season_folder, file_name)),
        });
    }

    if THEME_MEDIA_BASENAMES.contains(base_name.as_str()) && is_media_ext(&ext) {
        let file_name = path.file_name().unwrap_or("unknown");
        return Some(ShowAsset {
            source_path: path.to_path_buf(),
            asset_kind: ShowAssetKind::ThemeMusic,
            destination: show
                .relative_destination(&format!("{}/theme-music/{}", season_folder, file_name)),
        });
    }

    if is_media_ext(&ext) {
        let video_name = path.file_name().unwrap_or("").to_lowercase();
        if video_name.starts_with("sample")
            || video_name.contains("-sample")
            || video_name.contains(".sample")
            || video_name.contains("_sample")
            || video_name.contains("-trailer")
            || video_name.contains(".trailer")
            || video_name.contains("_trailer")
        {
            let file_name = path.file_name().unwrap_or("unknown");
            return Some(ShowAsset {
                source_path: path.to_path_buf(),
                asset_kind: ShowAssetKind::ExtrasFolder,
                destination: show.relative_destination(&format!("{}/{}", season_folder, file_name)),
            });
        }
    }

    None
}

fn is_media_ext(ext: &str) -> bool {
    VIDEO_EXTS.contains(ext) || AUDIO_EXTS.contains(ext)
}

impl TvShow {
    /// Build the show folder name (e.g. "Breaking Bad (2008) [tmdbid-1396]").
    pub fn show_folder_name(&self) -> String {
        crate::planner::build_show_folder_name(self)
    }

    /// Relative destination path for a show-level asset.
    ///
    /// Returns a path like "Show Name (Year) [tmdbid-123]/poster.jpg".
    /// Callers join this with `tv_root` to get the absolute destination.
    fn relative_destination(&self, relative_path: &str) -> Utf8PathBuf {
        let folder = self.show_folder_name();
        Utf8PathBuf::from(format!("{}/{}", folder, relative_path))
    }

    /// Resolve all asset destinations to absolute paths using the given TV root.
    pub fn resolve_asset_destinations(&mut self, tv_root: &str) {
        if tv_root.is_empty() {
            return;
        }
        let tv_root_path = Utf8PathBuf::from(tv_root);
        for asset in &mut self.show_assets {
            let dest = tv_root_path.join(&asset.destination);
            asset.destination = dest;
        }
        for season in &mut self.seasons {
            for asset in &mut season.season_assets {
                let dest = tv_root_path.join(&asset.destination);
                asset.destination = dest;
            }
        }
    }

    pub fn with_assets(mut self, assets: Vec<ShowAsset>) -> Self {
        self.show_assets = assets;
        self
    }

    pub fn with_season_assets(mut self, season_number: u16, assets: Vec<ShowAsset>) -> Self {
        if let Some(season) = self.seasons.iter_mut().find(|s| s.season_number == season_number) {
            season.season_assets = assets;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_show(title: &str, year: Option<u16>, source_root: &str) -> TvShow {
        TvShow {
            title: title.to_string(),
            year,
            tmdb_id: None,
            tvdb_id: None,
            imdb_id: None,
            source_root: Utf8PathBuf::from(source_root),
            seasons: vec![],
            show_assets: vec![],
            confidence: 50,
            reasons: vec![],
        }
    }

    #[test]
    fn discover_show_assets_finds_nfo() {
        let dir = tempfile::tempdir().unwrap();
        let show_dir = dir.path().join("Breaking Bad (2008)");
        fs::create_dir_all(&show_dir).unwrap();
        fs::write(show_dir.join("tvshow.nfo"), b"<tvshow></tvshow>").unwrap();

        let show = make_show("Breaking Bad", Some(2008), show_dir.to_string_lossy().as_ref());
        let assets = discover_show_assets(&show);

        assert!(assets.iter().any(|a| a.asset_kind == ShowAssetKind::Nfo));
    }

    #[test]
    fn discover_show_assets_finds_poster() {
        let dir = tempfile::tempdir().unwrap();
        let show_dir = dir.path().join("The Office (2005)");
        fs::create_dir_all(&show_dir).unwrap();
        fs::write(show_dir.join("poster.jpg"), b"").unwrap();

        let show = make_show("The Office", Some(2005), show_dir.to_string_lossy().as_ref());
        let assets = discover_show_assets(&show);

        assert!(assets.iter().any(|a| a.asset_kind == ShowAssetKind::Poster));
    }

    #[test]
    fn discover_show_assets_finds_backdrop() {
        let dir = tempfile::tempdir().unwrap();
        let show_dir = dir.path().join("Lost (2004)");
        fs::create_dir_all(&show_dir).unwrap();
        fs::write(show_dir.join("fanart.jpg"), b"").unwrap();

        let show = make_show("Lost", Some(2004), show_dir.to_string_lossy().as_ref());
        let assets = discover_show_assets(&show);

        assert!(assets.iter().any(|a| a.asset_kind == ShowAssetKind::Backdrop));
    }

    #[test]
    fn discover_show_assets_finds_theme_music_folder() {
        let dir = tempfile::tempdir().unwrap();
        let show_dir = dir.path().join("Friends (1994)");
        fs::create_dir_all(&show_dir).unwrap();
        let theme_dir = show_dir.join("theme-music");
        fs::create_dir_all(&theme_dir).unwrap();
        fs::write(theme_dir.join("opening.mp3"), b"").unwrap();

        let show = make_show("Friends", Some(1994), show_dir.to_string_lossy().as_ref());
        let assets = discover_show_assets(&show);

        assert!(assets.iter().any(|a| a.asset_kind == ShowAssetKind::ThemeMusic));
    }

    #[test]
    fn discover_show_assets_finds_season_poster() {
        let dir = tempfile::tempdir().unwrap();
        let show_dir = dir.path().join("Frasier (1993)");
        fs::create_dir_all(&show_dir).unwrap();
        let season_dir = show_dir.join("Season 01");
        fs::create_dir_all(&season_dir).unwrap();
        fs::write(season_dir.join("poster.jpg"), b"").unwrap();

        let show = make_show("Frasier", Some(1993), show_dir.to_string_lossy().as_ref());
        let assets = discover_show_assets(&show);

        assert!(assets.iter().any(|a| matches!(a.asset_kind, ShowAssetKind::SeasonPoster)));
    }

    #[test]
    fn tv_show_destination_root_includes_year_and_tmdb() {
        let show = TvShow {
            title: "Breaking Bad".to_string(),
            year: Some(2008),
            tmdb_id: Some("1396".to_string()),
            tvdb_id: None,
            imdb_id: None,
            source_root: Utf8PathBuf::from("/source/Breaking Bad"),
            seasons: vec![],
            show_assets: vec![],
            confidence: 50,
            reasons: vec![],
        };

        let folder = show.show_folder_name();
        assert!(folder.contains("Breaking Bad"));
        assert!(folder.contains("(2008)"));
        assert!(folder.contains("[tmdbid-1396]"));
    }

    #[test]
    fn tv_show_destination_root_without_year() {
        let show = make_show("Test Show", None, "/source/Test");
        let folder = show.show_folder_name();
        assert!(folder.contains("Test Show"));
        assert!(!folder.contains("("));
    }

    #[test]
    fn resolve_asset_destinations_applies_tv_root() {
        let dir = tempfile::tempdir().unwrap();
        let show_dir = dir.path().join("Test Show (2020)");
        fs::create_dir_all(&show_dir).unwrap();
        fs::write(show_dir.join("poster.jpg"), b"").unwrap();

        let mut show = make_show("Test Show", Some(2020), show_dir.to_string_lossy().as_ref());
        let assets = discover_show_assets(&show);
        show.show_assets = assets;

        show.resolve_asset_destinations("/tv");

        for asset in &show.show_assets {
            assert!(
                asset.destination.starts_with("/tv"),
                "Asset destination '{}' should start with /tv",
                asset.destination
            );
        }
    }
}
