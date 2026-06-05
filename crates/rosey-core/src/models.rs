use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Movie,
    Show,
    Episode,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaItem {
    pub kind: MediaKind,
    pub source_path: Utf8PathBuf,
    pub title: Option<String>,
    pub year: Option<u16>,
    pub season: Option<u16>,
    #[serde(default)]
    pub episodes: Vec<u16>,
    pub part: Option<u16>,
    pub date: Option<String>,
    #[serde(default)]
    pub sidecars: Vec<Utf8PathBuf>,
    #[serde(default)]
    pub nfo: BTreeMap<String, Option<String>>,
}

impl MediaItem {
    pub fn unknown(source_path: impl Into<Utf8PathBuf>) -> Self {
        Self {
            kind: MediaKind::Unknown,
            source_path: source_path.into(),
            title: None,
            year: None,
            season: None,
            episodes: Vec::new(),
            part: None,
            date: None,
            sidecars: Vec::new(),
            nfo: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentificationResult {
    pub item: MediaItem,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceBand {
    Green,
    Yellow,
    Red,
}

pub fn confidence_band(confidence: u8) -> ConfidenceBand {
    match confidence {
        70..=100 => ConfidenceBand::Green,
        40..=69 => ConfidenceBand::Yellow,
        _ => ConfidenceBand::Red,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub confidence: u8,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    Skip,
    Replace,
    KeepBoth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovePlan {
    #[serde(default)]
    pub destination_paths: Vec<Utf8PathBuf>,
    #[serde(default)]
    pub conflicts: Vec<Conflict>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    pub source: Utf8PathBuf,
    pub destination: Utf8PathBuf,
    pub policy: ConflictPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveResult {
    pub success: bool,
    #[serde(default)]
    pub moved: Vec<Utf8PathBuf>,
    #[serde(default)]
    pub skipped: Vec<Utf8PathBuf>,
    #[serde(default)]
    pub replaced: Vec<Utf8PathBuf>,
    #[serde(default)]
    pub kept_both: Vec<Utf8PathBuf>,
    pub rollback_performed: bool,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "source")]
pub enum PersonReference {
    Tmdb { id: String },
    Imdb { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonIdentity {
    pub tmdb_id: String,
    pub name: Option<String>,
    pub imdb_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonMovieCredit {
    pub tmdb_id: String,
    pub title: String,
    pub year: Option<u16>,
    pub release_date: Option<String>,
    pub character: Option<String>,
    pub order: Option<u32>,
    pub imdb_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryMovie {
    pub source_path: Utf8PathBuf,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub title: Option<String>,
    pub year: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryMovieMatchKind {
    TmdbId,
    ImdbId,
    TitleYear,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedPersonMovie {
    pub credit: PersonMovieCredit,
    pub library_movie: LibraryMovie,
    pub match_kind: LibraryMovieMatchKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingPersonMoviesReport {
    pub person: PersonIdentity,
    #[serde(default)]
    pub missing: Vec<PersonMovieCredit>,
    #[serde(default)]
    pub owned: Vec<OwnedPersonMovie>,
    pub library_movies_scanned: usize,
    pub credits_scanned: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TvShow {
    pub title: String,
    pub year: Option<u16>,
    pub tmdb_id: Option<String>,
    pub tvdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub source_root: Utf8PathBuf,
    pub seasons: Vec<TvSeason>,
    pub show_assets: Vec<ShowAsset>,
    pub confidence: u8,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TvSeason {
    pub season_number: u16,
    pub episodes: Vec<IdentifiedEpisode>,
    pub season_assets: Vec<ShowAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiedEpisode {
    pub item: MediaItem,
    pub score: Score,
    pub destination: Utf8PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowAsset {
    pub source_path: Utf8PathBuf,
    pub asset_kind: ShowAssetKind,
    pub destination: Utf8PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShowAssetKind {
    Poster,
    Backdrop,
    Banner,
    Logo,
    Thumb,
    Landscape,
    Nfo,
    ThemeMusic,
    ExtrasFolder,
    SeasonPoster,
    SeasonBackdrop,
    SeasonBanner,
    SeasonThumb,
    SeasonNfo,
    EpisodeThumb,
    EpisodeSubtitle,
    Other,
}

impl TvShow {
    pub fn total_episodes(&self) -> usize {
        self.seasons.iter().map(|s| s.episodes.len()).sum()
    }

    pub fn season_numbers(&self) -> Vec<u16> {
        let mut nums: Vec<u16> = self.seasons.iter().map(|s| s.season_number).collect();
        nums.sort();
        nums
    }
}
