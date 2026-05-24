use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join("AppData").join("Roaming")
        })
    } else {
        std::env::var("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        })
    };

    let config_dir = base.join("rosey");
    std::fs::create_dir_all(&config_dir).ok();
    config_dir.join("rosey.json")
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathsConfig {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub movies: String,
    #[serde(default)]
    pub tv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: u32,
    #[serde(default = "default_window_height")]
    pub height: u32,
    #[serde(default)]
    pub maximized: bool,
}

fn default_window_width() -> u32 {
    1200
}
fn default_window_height() -> u32 {
    800
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self { width: default_window_width(), height: default_window_height(), maximized: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplittersConfig {
    #[serde(default = "default_splitters_main")]
    pub main: Vec<u32>,
    #[serde(default = "default_splitters_vertical")]
    pub vertical: Vec<u32>,
}

fn default_splitters_main() -> Vec<u32> {
    vec![300, 900]
}
fn default_splitters_vertical() -> Vec<u32> {
    vec![600, 200]
}

impl Default for SplittersConfig {
    fn default() -> Self {
        Self { main: default_splitters_main(), vertical: default_splitters_vertical() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub splitters: SplittersConfig,
}

fn default_theme() -> String {
    "default".into()
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            window: WindowConfig::default(),
            splitters: SplittersConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    #[serde(default = "default_true")]
    pub dry_run: bool,
    #[serde(default = "default_true")]
    pub auto_select_green: bool,
    #[serde(default = "default_true")]
    pub confirm_delete: bool,
    #[serde(default)]
    pub conflict_policy: String,
    #[serde(default = "default_auto_delete_patterns")]
    pub auto_delete_patterns: Vec<String>,
}

fn default_true() -> bool {
    true
}
fn default_auto_delete_patterns() -> Vec<String> {
    vec!["sample.mkv".into(), "*.nfo".into(), "*.txt".into()]
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            dry_run: true,
            auto_select_green: true,
            confirm_delete: true,
            conflict_policy: "ask".into(),
            auto_delete_patterns: default_auto_delete_patterns(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanningConfig {
    #[serde(default = "default_concurrency_local")]
    pub concurrency_local: usize,
    #[serde(default = "default_two")]
    pub concurrency_network: usize,
    #[serde(default)]
    pub follow_symlinks: bool,
    #[serde(default)]
    pub enforce_one_media_per_folder: bool,
}

fn default_concurrency_local() -> usize {
    8
}
fn default_two() -> usize {
    2
}

impl Default for ScanningConfig {
    fn default() -> Self {
        Self {
            concurrency_local: default_concurrency_local(),
            concurrency_network: default_two(),
            follow_symlinks: false,
            enforce_one_media_per_folder: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub tmdb_api_key: String,
    #[serde(default = "default_en_us")]
    pub tmdb_language: String,
    #[serde(default = "default_us")]
    pub tmdb_region: String,
    #[serde(default)]
    pub tvdb_api_key: String,
    #[serde(default = "default_eng")]
    pub tvdb_language: String,
    #[serde(default = "default_thirty")]
    pub cache_ttl_days: u32,
}

fn default_en_us() -> String {
    "en-US".into()
}
fn default_us() -> String {
    "US".into()
}
fn default_eng() -> String {
    "eng".into()
}
fn default_thirty() -> u32 {
    30
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            tmdb_api_key: String::new(),
            tmdb_language: default_en_us(),
            tmdb_region: default_us(),
            tvdb_api_key: String::new(),
            tvdb_language: default_eng(),
            cache_ttl_days: default_thirty(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentificationConfig {
    #[serde(default)]
    pub use_online_providers: bool,
    #[serde(default = "default_thresholds")]
    pub confidence_thresholds: ConfidenceThresholds,
    #[serde(default = "default_true")]
    pub prefer_nfo_ids: bool,
    #[serde(default = "default_sixty")]
    pub minimum_movie_duration_minutes: u32,
    #[serde(default = "default_true")]
    pub movies_always_in_own_directory: bool,
    #[serde(default)]
    pub title_remove_segments: Vec<String>,
}

fn default_sixty() -> u32 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceThresholds {
    #[serde(default = "default_seventy")]
    pub green: u8,
    #[serde(default = "default_forty")]
    pub yellow: u8,
}

fn default_seventy() -> u8 {
    70
}
fn default_forty() -> u8 {
    40
}
fn default_thresholds() -> ConfidenceThresholds {
    ConfidenceThresholds { green: default_seventy(), yellow: default_forty() }
}

impl Default for IdentificationConfig {
    fn default() -> Self {
        Self {
            use_online_providers: false,
            confidence_thresholds: default_thresholds(),
            prefer_nfo_ids: true,
            minimum_movie_duration_minutes: default_sixty(),
            movies_always_in_own_directory: true,
            title_remove_segments: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_info")]
    pub level: String,
    #[serde(default)]
    pub file_path: String,
    #[serde(default = "default_ten")]
    pub max_file_size_mb: u32,
    #[serde(default = "default_five")]
    pub backup_count: u32,
    #[serde(default = "default_true")]
    pub redact_secrets: bool,
    #[serde(default)]
    pub log_to_console: bool,
}

fn default_info() -> String {
    "INFO".into()
}
fn default_ten() -> u32 {
    10
}
fn default_five() -> u32 {
    5
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_info(),
            file_path: String::new(),
            max_file_size_mb: default_ten(),
            backup_count: default_five(),
            redact_secrets: true,
            log_to_console: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoseyConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub ui: UIConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub scanning: ScanningConfig,
    #[serde(default)]
    pub identification: IdentificationConfig,
    #[serde(default)]
    pub providers: ProvidersConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

fn default_version() -> String {
    "1.0".into()
}

impl Default for RoseyConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            paths: PathsConfig::default(),
            ui: UIConfig::default(),
            behavior: BehaviorConfig::default(),
            scanning: ScanningConfig::default(),
            identification: IdentificationConfig::default(),
            providers: ProvidersConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

pub fn load_config() -> RoseyConfig {
    let path = config_path();
    if !path.exists() {
        return RoseyConfig::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            tracing::warn!("Failed to parse config from {}: {}", path.display(), e);
            RoseyConfig::default()
        }),
        Err(e) => {
            tracing::warn!("Failed to read config from {}: {}", path.display(), e);
            RoseyConfig::default()
        }
    }
}

pub fn save_config(config: &RoseyConfig) -> std::io::Result<()> {
    let path = config_path();
    let content = serde_json::to_string_pretty(config).map_err(std::io::Error::other)?;
    std::fs::write(&path, content)
}
