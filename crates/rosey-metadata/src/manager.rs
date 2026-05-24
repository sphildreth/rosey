use crate::cache::ProviderCache;
use crate::providers::{TmdbProvider, TvdbProvider};
use serde_json::Value;
use std::path::Path;

/// Manages metadata providers with caching.
///
/// Mirrors Python `ProviderManager` from `rosey/providers/manager.py`.
pub struct ProviderManager {
    pub cache: ProviderCache,
    pub enabled: bool,
    tmdb: Option<TmdbProvider>,
    tvdb: Option<TvdbProvider>,
}

impl ProviderManager {
    /// Create a new manager. Online lookups are disabled by default.
    pub fn new(
        cache_dir: impl AsRef<Path>,
        cache_ttl_days: u32,
        enabled: bool,
    ) -> decentdb::Result<Self> {
        let cache = ProviderCache::open(cache_dir, cache_ttl_days)?;
        Ok(Self { cache, enabled, tmdb: None, tvdb: None })
    }

    /// Configure the TMDB provider.
    pub fn configure_tmdb(
        &mut self,
        api_key: impl Into<String>,
        language: impl Into<String>,
        region: impl Into<String>,
    ) {
        self.tmdb = Some(TmdbProvider::with_options(
            api_key,
            language,
            region,
            std::time::Duration::from_secs(10),
        ));
    }

    /// Configure the TVDB provider.
    pub fn configure_tvdb(&mut self, api_key: impl Into<String>, language: impl Into<String>) {
        self.tvdb =
            Some(TvdbProvider::with_options(api_key, language, std::time::Duration::from_secs(10)));
    }

    /// Search for movies.
    pub async fn search_movie(
        &self,
        title: &str,
        year: Option<u16>,
        use_cache: bool,
    ) -> Vec<Value> {
        if !self.enabled || self.tmdb.is_none() {
            return Vec::new();
        }

        let cache_key = format!("{}_{}", title, year.map(|y| y.to_string()).unwrap_or_default());

        if use_cache {
            if let Some(cached) = self.cache.get("tmdb", "search_movie", &cache_key) {
                if let Some(arr) = cached.as_array() {
                    return arr.clone();
                }
            }
        }

        let tmdb = self.tmdb.as_ref().unwrap();
        let results = tmdb.search_movie(title, year).await;

        if use_cache {
            let value = Value::Array(results.clone());
            let _ = self.cache.set("tmdb", "search_movie", &cache_key, &value);
        }

        results
    }

    /// Search for TV shows.
    pub async fn search_tv(&self, title: &str, year: Option<u16>, use_cache: bool) -> Vec<Value> {
        if !self.enabled || self.tmdb.is_none() {
            return Vec::new();
        }

        let cache_key = format!("{}_{}", title, year.map(|y| y.to_string()).unwrap_or_default());

        if use_cache {
            if let Some(cached) = self.cache.get("tmdb", "search_tv", &cache_key) {
                if let Some(arr) = cached.as_array() {
                    return arr.clone();
                }
            }
        }

        let tmdb = self.tmdb.as_ref().unwrap();
        let results = tmdb.search_tv(title, year).await;

        if use_cache {
            let value = Value::Array(results.clone());
            let _ = self.cache.set("tmdb", "search_tv", &cache_key, &value);
        }

        results
    }

    /// Get movie details by ID.
    pub async fn get_movie_by_id(&self, movie_id: &str, use_cache: bool) -> Option<Value> {
        if !self.enabled || self.tmdb.is_none() {
            return None;
        }

        if use_cache {
            if let Some(cached) = self.cache.get("tmdb", "movie", movie_id) {
                return Some(cached);
            }
        }

        let tmdb = self.tmdb.as_ref().unwrap();
        let result = tmdb.get_movie_by_id(movie_id).await;

        if let Some(ref data) = result {
            if use_cache {
                let _ = self.cache.set("tmdb", "movie", movie_id, data);
            }
        }

        result
    }

    /// Get TV show details by ID.
    pub async fn get_tv_by_id(&self, tv_id: &str, use_cache: bool) -> Option<Value> {
        if !self.enabled || self.tmdb.is_none() {
            return None;
        }

        if use_cache {
            if let Some(cached) = self.cache.get("tmdb", "tv", tv_id) {
                return Some(cached);
            }
        }

        let tmdb = self.tmdb.as_ref().unwrap();
        let result = tmdb.get_tv_by_id(tv_id).await;

        if let Some(ref data) = result {
            if use_cache {
                let _ = self.cache.set("tmdb", "tv", tv_id, data);
            }
        }

        result
    }

    /// Get episode details.
    pub async fn get_episode(
        &self,
        tv_id: &str,
        season: u16,
        episode: u16,
        use_cache: bool,
    ) -> Option<Value> {
        if !self.enabled || self.tmdb.is_none() {
            return None;
        }

        let cache_key = format!("{tv_id}_s{season}e{episode}");

        if use_cache {
            if let Some(cached) = self.cache.get("tmdb", "episode", &cache_key) {
                return Some(cached);
            }
        }

        let tmdb = self.tmdb.as_ref().unwrap();
        let result = tmdb.get_episode(tv_id, season, episode).await;

        if let Some(ref data) = result {
            if use_cache {
                let _ = self.cache.set("tmdb", "episode", &cache_key, data);
            }
        }

        result
    }
}
