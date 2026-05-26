use crate::cache::ProviderCache;
use crate::providers::{TmdbProvider, TvdbProvider};
use rosey_core::{PersonIdentity, PersonMovieCredit};
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

    /// Search all configured movie providers.
    pub async fn search_movie_all(
        &self,
        title: &str,
        year: Option<u16>,
        use_cache: bool,
    ) -> Vec<(String, Value)> {
        if !self.enabled {
            return Vec::new();
        }

        let mut results = self
            .search_movie(title, year, use_cache)
            .await
            .into_iter()
            .map(|value| ("tmdb".to_string(), value))
            .collect::<Vec<_>>();

        if let Some(tvdb) = &self.tvdb {
            let cache_key =
                format!("{}_{}", title, year.map(|y| y.to_string()).unwrap_or_default());
            let tvdb_results = if use_cache {
                if let Some(cached) = self
                    .cache
                    .get("tvdb", "search_movie", &cache_key)
                    .and_then(|cached| cached.as_array().cloned())
                {
                    cached
                } else {
                    let values = tvdb.search_movie(title, year).await;
                    let _ = self.cache.set(
                        "tvdb",
                        "search_movie",
                        &cache_key,
                        &Value::Array(values.clone()),
                    );
                    values
                }
            } else {
                tvdb.search_movie(title, year).await
            };
            results.extend(tvdb_results.into_iter().map(|value| ("tvdb".to_string(), value)));
        }

        results
    }

    /// Search all configured TV providers.
    pub async fn search_tv_all(
        &self,
        title: &str,
        year: Option<u16>,
        use_cache: bool,
    ) -> Vec<(String, Value)> {
        if !self.enabled {
            return Vec::new();
        }

        let mut results = self
            .search_tv(title, year, use_cache)
            .await
            .into_iter()
            .map(|value| ("tmdb".to_string(), value))
            .collect::<Vec<_>>();

        if let Some(tvdb) = &self.tvdb {
            let cache_key =
                format!("{}_{}", title, year.map(|y| y.to_string()).unwrap_or_default());
            let tvdb_results = if use_cache {
                if let Some(cached) = self
                    .cache
                    .get("tvdb", "search_tv", &cache_key)
                    .and_then(|cached| cached.as_array().cloned())
                {
                    cached
                } else {
                    let values = tvdb.search_tv(title, year).await;
                    let _ = self.cache.set(
                        "tvdb",
                        "search_tv",
                        &cache_key,
                        &Value::Array(values.clone()),
                    );
                    values
                }
            } else {
                tvdb.search_tv(title, year).await
            };
            results.extend(tvdb_results.into_iter().map(|value| ("tvdb".to_string(), value)));
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

    /// Resolve a TMDB person from an IMDb name ID.
    pub async fn find_tmdb_person_by_imdb_id(
        &self,
        imdb_id: &str,
        use_cache: bool,
    ) -> Option<PersonIdentity> {
        if !self.enabled || self.tmdb.is_none() {
            return None;
        }

        let cache_key = imdb_id.trim();
        if cache_key.is_empty() {
            return None;
        }

        if use_cache {
            if let Some(cached) = self.cache.get("tmdb", "find_person_by_imdb", cache_key) {
                return parse_find_person_by_imdb(cache_key, &cached);
            }
        }

        let tmdb = self.tmdb.as_ref().unwrap();
        let result = tmdb.find_by_external_id(cache_key, "imdb_id").await;

        if let Some(ref data) = result {
            if use_cache {
                let _ = self.cache.set("tmdb", "find_person_by_imdb", cache_key, data);
            }
        }

        result.as_ref().and_then(|data| parse_find_person_by_imdb(cache_key, data))
    }

    /// Get cast movie credits for a TMDB person.
    pub async fn get_person_movie_credits(
        &self,
        person_id: &str,
        use_cache: bool,
    ) -> Vec<PersonMovieCredit> {
        if !self.enabled || self.tmdb.is_none() {
            return Vec::new();
        }

        let cache_key = person_id.trim();
        if cache_key.is_empty() {
            return Vec::new();
        }

        if use_cache {
            if let Some(cached) = self.cache.get("tmdb", "person_movie_credits", cache_key) {
                return parse_person_movie_credits(&cached);
            }
        }

        let tmdb = self.tmdb.as_ref().unwrap();
        let result = tmdb.get_person_movie_credits(cache_key).await;

        if let Some(ref data) = result {
            if use_cache {
                let _ = self.cache.set("tmdb", "person_movie_credits", cache_key, data);
            }
        }

        result.as_ref().map(parse_person_movie_credits).unwrap_or_default()
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

fn parse_find_person_by_imdb(imdb_id: &str, value: &Value) -> Option<PersonIdentity> {
    let person = value.get("person_results")?.as_array()?.first()?;
    let tmdb_id = value_to_string(person.get("id")?)?;
    let name = person
        .get("name")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToString::to_string);

    Some(PersonIdentity { tmdb_id, name, imdb_id: Some(imdb_id.to_string()) })
}

fn parse_person_movie_credits(value: &Value) -> Vec<PersonMovieCredit> {
    value
        .get("cast")
        .and_then(|cast| cast.as_array())
        .into_iter()
        .flatten()
        .filter_map(parse_person_movie_credit)
        .collect()
}

fn parse_person_movie_credit(value: &Value) -> Option<PersonMovieCredit> {
    let tmdb_id = value_to_string(value.get("id")?)?;
    let title = ["title", "original_title"]
        .into_iter()
        .find_map(|key| value.get(key).and_then(|value| value.as_str()))
        .map(str::trim)
        .filter(|title| !title.is_empty())?
        .to_string();
    let release_date = value
        .get("release_date")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|date| !date.is_empty())
        .map(ToString::to_string);
    let year = release_date.as_deref().and_then(year_from_date);
    let character = value
        .get("character")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|character| !character.is_empty())
        .map(ToString::to_string);
    let order = value
        .get("order")
        .and_then(|value| value.as_u64())
        .and_then(|order| u32::try_from(order).ok());

    Some(PersonMovieCredit { tmdb_id, title, year, release_date, character, order, imdb_id: None })
}

fn value_to_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(ToString::to_string)
        .or_else(|| value.as_u64().map(|id| id.to_string()))
        .filter(|id| !id.is_empty())
}

fn year_from_date(date: &str) -> Option<u16> {
    date.get(0..4)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cached_find_resolves_imdb_name_to_tmdb_person() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut manager = ProviderManager::new(temp_dir.path(), 30, true).unwrap();
        manager.configure_tmdb("test-key", "en-US", "US");
        manager
            .cache
            .set(
                "tmdb",
                "find_person_by_imdb",
                "nm0000702",
                &json!({
                    "person_results": [
                        { "id": 368, "name": "Marlon Brando" }
                    ]
                }),
            )
            .unwrap();

        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let person =
            runtime.block_on(manager.find_tmdb_person_by_imdb_id("nm0000702", true)).unwrap();

        assert_eq!(person.tmdb_id, "368");
        assert_eq!(person.name.as_deref(), Some("Marlon Brando"));
        assert_eq!(person.imdb_id.as_deref(), Some("nm0000702"));
    }

    #[test]
    fn cached_person_movie_credits_parse_cast_movies() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut manager = ProviderManager::new(temp_dir.path(), 30, true).unwrap();
        manager.configure_tmdb("test-key", "en-US", "US");
        manager
            .cache
            .set(
                "tmdb",
                "person_movie_credits",
                "368",
                &json!({
                    "id": 368,
                    "cast": [
                        {
                            "id": 238,
                            "title": "The Godfather",
                            "release_date": "1972-03-14",
                            "character": "Don Vito Corleone",
                            "order": 0
                        }
                    ],
                    "crew": [
                        { "id": 1, "title": "Crew Movie" }
                    ]
                }),
            )
            .unwrap();

        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let credits = runtime.block_on(manager.get_person_movie_credits("368", true));

        assert_eq!(credits.len(), 1);
        assert_eq!(credits[0].tmdb_id, "238");
        assert_eq!(credits[0].title, "The Godfather");
        assert_eq!(credits[0].year, Some(1972));
        assert_eq!(credits[0].character.as_deref(), Some("Don Vito Corleone"));
    }
}
