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
