use camino::{Utf8Path, Utf8PathBuf};
use rosey_core::{
    config_path, extract_tmdb_id_from_path, identify_file_with_options, IdentificationResult,
    IdentifyOptions, MediaKind, RoseyConfig,
};
use serde_json::Value;
use std::path::PathBuf;

use crate::ProviderManager;

/// Identifies files with optional online provider enrichment.
///
/// Reuse one `BatchIdentifier` across a batch of files so the provider cache
/// database and provider clients are opened once instead of per file.
pub struct BatchIdentifier {
    config: RoseyConfig,
    manager: Option<ProviderManager>,
    manager_error: Option<String>,
}

impl BatchIdentifier {
    /// Build an identifier for a batch. Provider setup failures are deferred to
    /// each `identify` call so offline identification keeps working.
    pub fn new(config: &RoseyConfig) -> Self {
        if !config.identification.use_online_providers {
            return Self { config: config.clone(), manager: None, manager_error: None };
        }

        let has_tmdb = !config.providers.tmdb_api_key.trim().is_empty();
        let has_tvdb = !config.providers.tvdb_api_key.trim().is_empty();
        if !has_tmdb && !has_tvdb {
            return Self { config: config.clone(), manager: None, manager_error: None };
        }

        match build_manager(config, has_tmdb, has_tvdb) {
            Ok(manager) => {
                Self { config: config.clone(), manager: Some(manager), manager_error: None }
            }
            Err(error) => {
                Self { config: config.clone(), manager: None, manager_error: Some(error) }
            }
        }
    }

    /// Identify one file, enriching the offline result with provider data when
    /// online providers are configured.
    pub async fn identify(
        &self,
        path: &Utf8Path,
        options: IdentifyOptions,
    ) -> IdentificationResult {
        let mut result = identify_file_with_options(path, &self.config, options);

        let Some(manager) = self.manager.as_ref() else {
            if let Some(error) = &self.manager_error {
                result.errors.push(error.clone());
            }
            return result;
        };

        if manager.has_tmdb() {
            if let Some(tmdb_id) = extract_tmdb_id_from_path(path.as_str()) {
                if let Some(movie) = manager.get_movie_by_id(&tmdb_id, true).await {
                    if movie.get("id").is_some() {
                        apply_tmdb_movie(&mut result, &tmdb_id, &movie);
                        return result;
                    }
                }

                if let Some(tv) = manager.get_tv_by_id(&tmdb_id, true).await {
                    if tv.get("id").is_some() {
                        apply_tmdb_tv(&mut result, &tmdb_id, &tv);
                        return result;
                    }
                }

                result.reasons.push(format!("TMDB ID {tmdb_id} from path - API lookup failed"));
            }
        }

        auto_identify_movie_from_search(&mut result, manager).await;
        auto_identify_tv_from_search(&mut result, manager).await;
        result
    }
}

pub async fn identify_file_with_metadata(
    path: &Utf8Path,
    config: &RoseyConfig,
    options: IdentifyOptions,
) -> IdentificationResult {
    BatchIdentifier::new(config).identify(path, options).await
}

fn build_manager(
    config: &RoseyConfig,
    has_tmdb: bool,
    has_tvdb: bool,
) -> Result<ProviderManager, String> {
    let mut manager = ProviderManager::new(
        metadata_cache_dir(),
        config.providers.cache_ttl_days,
        config.identification.use_online_providers,
    )
    .map_err(|_| "Failed to initialize metadata provider cache".to_string())?;

    if has_tmdb {
        manager.configure_tmdb(
            config.providers.tmdb_api_key.clone(),
            config.providers.tmdb_language.clone(),
            config.providers.tmdb_region.clone(),
        );
    }
    if has_tvdb {
        manager.configure_tvdb(
            config.providers.tvdb_api_key.clone(),
            config.providers.tvdb_language.clone(),
        );
    }

    Ok(manager)
}

fn metadata_cache_dir() -> PathBuf {
    config_path()
        .parent()
        .map(|path| path.join("cache"))
        .unwrap_or_else(|| PathBuf::from(".").join("cache"))
}

fn apply_tmdb_movie(result: &mut IdentificationResult, tmdb_id: &str, movie: &Value) {
    result.item.kind = MediaKind::Movie;
    result.item.title = movie
        .get("title")
        .and_then(|title| title.as_str())
        .filter(|title| !title.is_empty())
        .map(ToString::to_string)
        .or_else(|| result.item.title.clone());
    result.item.year = movie
        .get("release_date")
        .and_then(|date| year_from_date(date.as_str()?))
        .or(result.item.year);
    result.item.season = None;
    result.item.episodes.clear();
    result.item.date = None;
    result.item.nfo.insert("tmdbid".to_string(), Some(tmdb_id.to_string()));
    result.item.nfo.insert("_source".to_string(), Some("identification".to_string()));
    if let Some(title) = result.item.title.clone() {
        result.item.nfo.insert("title".to_string(), Some(title));
    }
    result.reasons.push(format!("TMDB ID {tmdb_id} from path - definitive movie identification"));
}

#[derive(Debug, Clone)]
struct ProviderMatchCandidate {
    provider: String,
    id: String,
    title: String,
    year: Option<u16>,
}

async fn auto_identify_movie_from_search(
    result: &mut IdentificationResult,
    manager: &ProviderManager,
) {
    if result.item.kind != MediaKind::Movie
        || result.item.nfo.contains_key("tmdbid")
        || result.item.nfo.contains_key("tvdbid")
    {
        return;
    }

    let Some(title) = parsed_search_title(result) else {
        return;
    };
    let Some(year) = result.item.year else {
        return;
    };

    let matches = manager
        .search_movie_all(title, Some(year), true)
        .await
        .into_iter()
        .filter_map(|(provider, value)| provider_match_candidate(provider, value))
        .collect::<Vec<_>>();

    let selected = if matches.len() == 1 {
        matches.into_iter().next().map(|matched| {
            (matched, "single provider result matched parsed movie search".to_string())
        })
    } else {
        matches.into_iter().find(|matched| exact_movie_match(title, year, matched)).map(|matched| {
            (matched, "provider result exactly matched parsed title/year".to_string())
        })
    };

    let Some((matched, reason)) = selected else {
        return;
    };

    apply_provider_movie(result, matched, reason);
}

async fn auto_identify_tv_from_search(
    result: &mut IdentificationResult,
    manager: &ProviderManager,
) {
    if !matches!(result.item.kind, MediaKind::Episode | MediaKind::Show)
        || result.item.nfo.contains_key("tmdbid")
        || result.item.nfo.contains_key("tvdbid")
    {
        return;
    }

    let Some(title) = parsed_search_title(result) else {
        return;
    };
    let year = result.item.year;

    let matches = manager
        .search_tv_all(title, year, true)
        .await
        .into_iter()
        .filter_map(|(provider, value)| provider_match_candidate(provider, value))
        .collect::<Vec<_>>();

    // A wrong show pick would organize episodes into the wrong series, so the
    // no-year case only auto-selects an unambiguous single result.
    let selected = if matches.len() == 1 {
        matches.into_iter().next().map(|matched| {
            (matched, "single provider result matched parsed show search".to_string())
        })
    } else if year.is_some() {
        matches.into_iter().find(|matched| exact_show_match(title, year, matched)).map(|matched| {
            (matched, "provider result exactly matched parsed show title/year".to_string())
        })
    } else {
        None
    };

    let Some((matched, reason)) = selected else {
        return;
    };

    apply_provider_tv(result, matched, reason);
}

fn parsed_search_title(result: &IdentificationResult) -> Option<&str> {
    result.item.title.as_deref().filter(|title| !title.trim().is_empty())
}

fn provider_match_candidate(provider: String, value: Value) -> Option<ProviderMatchCandidate> {
    let id = provider_id(&value)?;
    let title = provider_title(&value)?;
    let year = provider_year(&value);
    Some(ProviderMatchCandidate { provider, id, title, year })
}

fn apply_provider_movie(
    result: &mut IdentificationResult,
    matched: ProviderMatchCandidate,
    reason: String,
) {
    result.item.kind = MediaKind::Movie;
    result.item.title = Some(matched.title.clone());
    result.item.year = matched.year.or(result.item.year);
    result.item.season = None;
    result.item.episodes.clear();
    result.item.date = None;

    let id_key = if matched.provider == "tvdb" { "tvdbid" } else { "tmdbid" };
    result.item.nfo.insert(id_key.to_string(), Some(matched.id.clone()));
    result.item.nfo.insert("title".to_string(), Some(matched.title.clone()));
    result.item.nfo.insert("_source".to_string(), Some("identification".to_string()));
    result.item.nfo.insert("_provider".to_string(), Some(matched.provider.clone()));
    result.reasons.push(format!(
        "{} movie search selected {} ({}) - {reason}",
        matched.provider.to_uppercase(),
        matched.title,
        matched.year.map(|year| year.to_string()).unwrap_or_else(|| "?".to_string())
    ));
}

fn apply_provider_tv(
    result: &mut IdentificationResult,
    matched: ProviderMatchCandidate,
    reason: String,
) {
    result.item.title = Some(matched.title.clone());
    result.item.year = matched.year.or(result.item.year);

    let id_key = if matched.provider == "tvdb" { "tvdbid" } else { "tmdbid" };
    result.item.nfo.insert(id_key.to_string(), Some(matched.id.clone()));
    result.item.nfo.insert("title".to_string(), Some(matched.title.clone()));
    result.item.nfo.insert("_source".to_string(), Some("identification".to_string()));
    result.item.nfo.insert("_provider".to_string(), Some(matched.provider.clone()));
    result.reasons.push(format!(
        "{} show search selected {} ({}) - {reason}",
        matched.provider.to_uppercase(),
        matched.title,
        matched.year.map(|year| year.to_string()).unwrap_or_else(|| "?".to_string())
    ));
}

fn provider_id(value: &Value) -> Option<String> {
    ["id", "tvdb_id", "thetvdb_id"]
        .into_iter()
        .find_map(|key| value.get(key))
        .and_then(value_to_string)
}

fn provider_title(value: &Value) -> Option<String> {
    ["title", "name"]
        .into_iter()
        .find_map(|key| value.get(key))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(ToString::to_string)
}

fn provider_year(value: &Value) -> Option<u16> {
    ["release_date", "first_air_date", "firstAired", "aired"]
        .into_iter()
        .find_map(|key| value.get(key).and_then(|value| value.as_str()).and_then(year_from_date))
        .or_else(|| {
            ["year", "release_year", "first_air_year"]
                .into_iter()
                .find_map(|key| value.get(key))
                .and_then(value_to_year)
        })
}

fn value_to_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(ToString::to_string)
        .or_else(|| value.as_u64().map(|id| id.to_string()))
        .filter(|id| !id.is_empty())
}

fn value_to_year(value: &Value) -> Option<u16> {
    if let Some(year) = value.as_u64().and_then(|year| u16::try_from(year).ok()) {
        return Some(year);
    }
    value.as_str().and_then(|year| year.get(0..4)?.parse().ok())
}

fn exact_movie_match(title: &str, year: u16, matched: &ProviderMatchCandidate) -> bool {
    normalize_title_for_match(title) == normalize_title_for_match(&matched.title)
        && matched.year == Some(year)
}

fn exact_show_match(title: &str, year: Option<u16>, matched: &ProviderMatchCandidate) -> bool {
    if normalize_title_for_match(title) != normalize_title_for_match(&matched.title) {
        return false;
    }

    match (year, matched.year) {
        (Some(parsed), Some(provider)) => parsed == provider,
        _ => true,
    }
}

fn normalize_title_for_match(title: &str) -> String {
    title.chars().filter(|c| c.is_ascii_alphanumeric()).flat_map(char::to_lowercase).collect()
}

fn apply_tmdb_tv(result: &mut IdentificationResult, tmdb_id: &str, tv: &Value) {
    result.item.kind = MediaKind::Episode;
    result.item.title = tv
        .get("name")
        .and_then(|title| title.as_str())
        .filter(|title| !title.is_empty())
        .map(ToString::to_string)
        .or_else(|| result.item.title.clone());
    result.item.year = tv
        .get("first_air_date")
        .and_then(|date| year_from_date(date.as_str()?))
        .or(result.item.year);
    result.item.nfo.insert("tmdbid".to_string(), Some(tmdb_id.to_string()));
    result.item.nfo.insert("_source".to_string(), Some("identification".to_string()));
    if let Some(title) = result.item.title.clone() {
        result.item.nfo.insert("title".to_string(), Some(title));
    }
    result.reasons.push(format!("TMDB ID {tmdb_id} from path - definitive TV show identification"));
}

fn year_from_date(date: &str) -> Option<u16> {
    date.get(0..4)?.parse().ok()
}

#[allow(dead_code)]
fn _assert_utf8_pathbuf_send(_: Utf8PathBuf) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProviderCache;
    use rosey_core::{score_identification_result, RoseyConfig};
    use serde_json::json;
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[cfg(not(windows))]
    fn config_env_key() -> &'static str {
        "XDG_CONFIG_HOME"
    }

    #[cfg(windows)]
    fn config_env_key() -> &'static str {
        "APPDATA"
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn set_env_var(key: &'static str, value: &std::path::Path) -> EnvVarGuard {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        EnvVarGuard { key, previous }
    }

    fn provider_enabled_config() -> RoseyConfig {
        let mut config = RoseyConfig::default();
        config.identification.use_online_providers = true;
        config.identification.movies_always_in_own_directory = false;
        config.providers.tmdb_api_key = "test-key".into();
        config
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
    }

    #[tokio::test]
    async fn disabled_metadata_extracts_path_tmdb_id() {
        let mut config = RoseyConfig::default();
        config.identification.use_online_providers = false;
        config.identification.movies_always_in_own_directory = false;

        let result = identify_file_with_metadata(
            Utf8Path::new("/media/[tmdbid-603] The Matrix (1999).mkv"),
            &config,
            IdentifyOptions { skip_duration: true },
        )
        .await;

        assert_eq!(result.item.kind, MediaKind::Movie);
        assert_eq!(result.item.title.as_deref(), Some("The Matrix"));
        assert_eq!(result.item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("603"));
        assert!(result.reasons.iter().any(|r| r.contains("TMDB ID from directory name")));
    }

    #[test]
    fn provider_cache_confirms_path_tmdb_movie() {
        let _guard = env_lock();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let _env = set_env_var(config_env_key(), temp_dir.path());
        let cache = ProviderCache::open(metadata_cache_dir(), 30).unwrap();
        cache
            .set(
                "tmdb",
                "movie",
                "603",
                &json!({
                    "id": 603,
                    "title": "The Matrix",
                    "release_date": "1999-03-31"
                }),
            )
            .unwrap();

        let mut config = provider_enabled_config();
        config.identification.movies_always_in_own_directory = false;

        let runtime = runtime();
        let result = runtime.block_on(identify_file_with_metadata(
            Utf8Path::new("/media/[tmdbid-603] The Matrix (1999).mkv"),
            &config,
            IdentifyOptions { skip_duration: true },
        ));
        let score = score_identification_result(&result);

        assert_eq!(result.item.kind, MediaKind::Movie);
        assert_eq!(result.item.title.as_deref(), Some("The Matrix"));
        assert_eq!(result.item.year, Some(1999));
        assert_eq!(result.item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("603"));
        assert!(score.confidence >= 70);
    }

    #[test]
    fn provider_search_auto_selects_single_movie_result() {
        let _guard = env_lock();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let _env = set_env_var(config_env_key(), temp_dir.path());
        let cache = ProviderCache::open(metadata_cache_dir(), 30).unwrap();
        cache
            .set(
                "tmdb",
                "search_movie",
                "Carry On Regardless_1961",
                &json!([
                    {
                        "id": 12345,
                        "title": "Carry On Regardless",
                        "release_date": "1961-03-31"
                    }
                ]),
            )
            .unwrap();

        let config = provider_enabled_config();

        let runtime = runtime();
        let result = runtime.block_on(identify_file_with_metadata(
            Utf8Path::new("/media/Carry.On.Regardless.1961.1080p.WEBRip.mkv"),
            &config,
            IdentifyOptions { skip_duration: true },
        ));

        assert_eq!(result.item.kind, MediaKind::Movie);
        assert_eq!(result.item.title.as_deref(), Some("Carry On Regardless"));
        assert_eq!(result.item.year, Some(1961));
        assert_eq!(result.item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("12345"));
    }

    #[test]
    fn provider_search_auto_selects_exact_movie_result() {
        let _guard = env_lock();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let _env = set_env_var(config_env_key(), temp_dir.path());
        let cache = ProviderCache::open(metadata_cache_dir(), 30).unwrap();
        cache
            .set(
                "tmdb",
                "search_movie",
                "Problemista_2023",
                &json!([
                    {
                        "id": 1,
                        "title": "Problem Child",
                        "release_date": "1990-07-27"
                    },
                    {
                        "id": 2,
                        "title": "Problemista",
                        "release_date": "2023-03-13"
                    }
                ]),
            )
            .unwrap();

        let config = provider_enabled_config();

        let runtime = runtime();
        let result = runtime.block_on(identify_file_with_metadata(
            Utf8Path::new("/media/Problemista.2023.1080p.WEBRip.x265.10bit.AAC5.1.mkv"),
            &config,
            IdentifyOptions { skip_duration: true },
        ));

        assert_eq!(result.item.title.as_deref(), Some("Problemista"));
        assert_eq!(result.item.year, Some(2023));
        assert_eq!(result.item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("2"));
    }

    #[test]
    fn provider_search_auto_selects_single_show_result() {
        let _guard = env_lock();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let _env = set_env_var(config_env_key(), temp_dir.path());
        let cache = ProviderCache::open(metadata_cache_dir(), 30).unwrap();
        cache
            .set(
                "tmdb",
                "search_tv",
                "Firefly_",
                &json!([
                    {
                        "id": 17975,
                        "name": "Firefly",
                        "first_air_date": "2002-12-20"
                    }
                ]),
            )
            .unwrap();

        let config = provider_enabled_config();

        let runtime = runtime();
        let result = runtime.block_on(identify_file_with_metadata(
            Utf8Path::new("/media/Firefly/Season 01/Firefly - S01E01 - Serenity.mkv"),
            &config,
            IdentifyOptions { skip_duration: true },
        ));

        assert_eq!(result.item.kind, MediaKind::Episode);
        assert_eq!(result.item.title.as_deref(), Some("Firefly"));
        assert_eq!(result.item.season, Some(1));
        assert_eq!(result.item.episodes, vec![1]);
        assert_eq!(result.item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("17975"));
        assert!(result.reasons.iter().any(|r| r.contains("show search selected Firefly")));
    }

    #[test]
    fn provider_search_auto_selects_exact_show_result() {
        let _guard = env_lock();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let _env = set_env_var(config_env_key(), temp_dir.path());
        let cache = ProviderCache::open(metadata_cache_dir(), 30).unwrap();
        cache
            .set(
                "tmdb",
                "search_tv",
                "Battlestar Galactica_2004",
                &json!([
                    {
                        "id": 11559,
                        "name": "Battlestar Galactica",
                        "first_air_date": "1978-09-17"
                    },
                    {
                        "id": 19783,
                        "name": "Battlestar Galactica",
                        "first_air_date": "2004-10-18"
                    }
                ]),
            )
            .unwrap();

        let config = provider_enabled_config();

        let runtime = runtime();
        let result = runtime.block_on(identify_file_with_metadata(
            Utf8Path::new(
                "/media/Battlestar Galactica (2004)/Season 02/Battlestar Galactica - S02E01.mkv",
            ),
            &config,
            IdentifyOptions { skip_duration: true },
        ));

        assert_eq!(result.item.kind, MediaKind::Episode);
        assert_eq!(result.item.year, Some(2004));
        assert_eq!(result.item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("19783"));
    }

    #[test]
    fn provider_search_rejects_ambiguous_show_without_year() {
        let _guard = env_lock();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let _env = set_env_var(config_env_key(), temp_dir.path());
        let cache = ProviderCache::open(metadata_cache_dir(), 30).unwrap();
        cache
            .set(
                "tmdb",
                "search_tv",
                "Battlestar Galactica_",
                &json!([
                    {
                        "id": 11559,
                        "name": "Battlestar Galactica",
                        "first_air_date": "1978-09-17"
                    },
                    {
                        "id": 19783,
                        "name": "Battlestar Galactica",
                        "first_air_date": "2004-10-18"
                    }
                ]),
            )
            .unwrap();

        let config = provider_enabled_config();

        let runtime = runtime();
        let result = runtime.block_on(identify_file_with_metadata(
            Utf8Path::new(
                "/media/Battlestar Galactica/Season 02/Battlestar Galactica - S02E01.mkv",
            ),
            &config,
            IdentifyOptions { skip_duration: true },
        ));

        assert_eq!(result.item.kind, MediaKind::Episode);
        assert!(!result.item.nfo.contains_key("tmdbid"));
        assert!(result.reasons.iter().all(|r| !r.contains("show search selected")));
    }

    #[test]
    fn extracts_year_from_provider_dates() {
        assert_eq!(year_from_date("1999-03-31"), Some(1999));
        assert_eq!(year_from_date("bad"), None);
    }
}
