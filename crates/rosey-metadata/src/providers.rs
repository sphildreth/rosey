use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Token-bucket rate limiter.
///
/// Mirrors Python `RateLimiter` from `rosey/providers/tmdb.py`.
pub struct RateLimiter {
    max_rps: usize,
    last_request_times: Arc<Mutex<VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn new(max_rps: usize) -> Self {
        Self { max_rps, last_request_times: Arc::new(Mutex::new(VecDeque::with_capacity(max_rps))) }
    }

    /// Acquire a token, blocking if necessary.
    pub async fn acquire(&self) {
        let mut times = self.last_request_times.lock().await;
        let now = Instant::now();

        // Remove timestamps older than 1 second
        while let Some(front) = times.front() {
            if now.duration_since(*front) >= Duration::from_secs(1) {
                times.pop_front();
            } else {
                break;
            }
        }

        // If we've hit the limit, wait
        if times.len() >= self.max_rps {
            let oldest = times.front().unwrap();
            let sleep_time = Duration::from_secs(1).saturating_sub(now.duration_since(*oldest));
            if sleep_time > Duration::ZERO {
                drop(times);
                tokio::time::sleep(sleep_time).await;
                // Re-acquire lock and clean up again
                let mut times = self.last_request_times.lock().await;
                let now = Instant::now();
                while let Some(front) = times.front() {
                    if now.duration_since(*front) >= Duration::from_secs(1) {
                        times.pop_front();
                    } else {
                        break;
                    }
                }
                times.push_back(Instant::now());
                return;
            }
        }

        times.push_back(now);
    }
}

/// TMDB metadata provider with rate limiting.
///
/// Mirrors Python `TMDBProvider` from `rosey/providers/tmdb.py`.
pub struct TmdbProvider {
    api_key: String,
    language: String,
    region: String,
    client: reqwest::Client,
    rate_limiter: RateLimiter,
}

impl TmdbProvider {
    const BASE_URL: &'static str = "https://api.themoviedb.org/3";
    const MAX_RPS: usize = 4;

    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_options(api_key, "en-US", "US", Duration::from_secs(10))
    }

    pub fn with_options(
        api_key: impl Into<String>,
        language: impl Into<String>,
        region: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        let client =
            reqwest::Client::builder().timeout(timeout).build().expect("valid reqwest client");

        Self {
            api_key: api_key.into(),
            language: language.into(),
            region: region.into(),
            client,
            rate_limiter: RateLimiter::new(Self::MAX_RPS),
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Option<T> {
        self._request(endpoint, params, 0).await
    }

    async fn _request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
        retry_count: u8,
    ) -> Option<T> {
        self.rate_limiter.acquire().await;

        let mut query: Vec<(&str, &str)> =
            vec![("api_key", &self.api_key), ("language", &self.language)];
        query.extend_from_slice(params);

        let url = format!("{}{}", Self::BASE_URL, endpoint);

        match self.client.get(&url).query(&query).send().await {
            Ok(resp) => {
                if resp.status() == 429 && retry_count == 0 {
                    // Rate limited — simple backoff then retry once
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    return Box::pin(self._request(endpoint, params, retry_count + 1)).await;
                }
                resp.json::<T>().await.ok()
            }
            Err(_) => None,
        }
    }

    /// Search for movies by title and optional year.
    pub async fn search_movie(&self, title: &str, year: Option<u16>) -> Vec<serde_json::Value> {
        let mut params = vec![("query", title), ("region", &self.region)];
        let year_str;
        if let Some(y) = year {
            year_str = y.to_string();
            params.push(("year", &year_str));
        }

        let result: Option<serde_json::Value> = self.request("/search/movie", &params).await;
        result
            .and_then(|v| v.get("results").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    }

    /// Search for TV shows by title and optional year.
    pub async fn search_tv(&self, title: &str, year: Option<u16>) -> Vec<serde_json::Value> {
        let mut params = vec![("query", title)];
        let year_str;
        if let Some(y) = year {
            year_str = y.to_string();
            params.push(("first_air_date_year", &year_str));
        }

        let result: Option<serde_json::Value> = self.request("/search/tv", &params).await;
        result
            .and_then(|v| v.get("results").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    }

    /// Get movie details by ID.
    pub async fn get_movie_by_id(&self, movie_id: &str) -> Option<serde_json::Value> {
        self.request(&format!("/movie/{movie_id}"), &[]).await
    }

    /// Get TV show details by ID.
    pub async fn get_tv_by_id(&self, tv_id: &str) -> Option<serde_json::Value> {
        self.request(&format!("/tv/{tv_id}"), &[]).await
    }

    /// Get episode details.
    pub async fn get_episode(
        &self,
        tv_id: &str,
        season: u16,
        episode: u16,
    ) -> Option<serde_json::Value> {
        self.request(&format!("/tv/{tv_id}/season/{season}/episode/{episode}"), &[]).await
    }
}

/// TVDB metadata provider with JWT authentication.
///
/// Mirrors Python `TVDBProvider` from `rosey/providers/tvdb.py`.
pub struct TvdbProvider {
    api_key: String,
    _language: String,
    client: reqwest::Client,
    token: Arc<std::sync::Mutex<Option<String>>>,
    token_expires: Arc<std::sync::Mutex<Instant>>,
}

impl TvdbProvider {
    const BASE_URL: &'static str = "https://api4.thetvdb.com/v4";

    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_options(api_key, "eng", Duration::from_secs(10))
    }

    pub fn with_options(
        api_key: impl Into<String>,
        language: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        let client =
            reqwest::Client::builder().timeout(timeout).build().expect("valid reqwest client");

        Self {
            api_key: api_key.into(),
            _language: language.into(),
            client,
            token: Arc::new(std::sync::Mutex::new(None)),
            token_expires: Arc::new(std::sync::Mutex::new(Instant::now())),
        }
    }

    async fn ensure_token(&self) -> bool {
        {
            let token = self.token.lock().unwrap();
            let expires = self.token_expires.lock().unwrap();
            if token.is_some() && Instant::now() < *expires {
                return true;
            }
        }

        match self
            .client
            .post(format!("{}/login", Self::BASE_URL))
            .json(&serde_json::json!({ "apikey": self.api_key }))
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(t) =
                        body.get("data").and_then(|d| d.get("token")).and_then(|t| t.as_str())
                    {
                        let mut token = self.token.lock().unwrap();
                        let mut expires = self.token_expires.lock().unwrap();
                        *token = Some(t.to_string());
                        // Refresh 1 hour before 30-day expiry
                        *expires = Instant::now() + Duration::from_secs(30 * 24 * 3600 - 3600);
                        return true;
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Option<T> {
        if !self.ensure_token().await {
            return None;
        }

        let token = self.token.lock().unwrap().clone().unwrap_or_default();
        let url = format!("{}{}", Self::BASE_URL, endpoint);

        match self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .query(&params)
            .send()
            .await
        {
            Ok(resp) => resp.json::<T>().await.ok(),
            Err(_) => None,
        }
    }

    /// Search for movies by title and optional year.
    pub async fn search_movie(&self, title: &str, year: Option<u16>) -> Vec<serde_json::Value> {
        let mut params = vec![("query", title), ("type", "movie")];
        let year_str;
        if let Some(y) = year {
            year_str = y.to_string();
            params.push(("year", &year_str));
        }

        let result: Option<serde_json::Value> = self.request("/search", &params).await;
        result
            .and_then(|v| v.get("data").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    }

    /// Search for TV shows by title and optional year.
    pub async fn search_tv(&self, title: &str, year: Option<u16>) -> Vec<serde_json::Value> {
        let mut params = vec![("query", title), ("type", "series")];
        let year_str;
        if let Some(y) = year {
            year_str = y.to_string();
            params.push(("year", &year_str));
        }

        let result: Option<serde_json::Value> = self.request("/search", &params).await;
        result
            .and_then(|v| v.get("data").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    }

    /// Get movie details by ID.
    pub async fn get_movie_by_id(&self, movie_id: &str) -> Option<serde_json::Value> {
        let result: Option<serde_json::Value> =
            self.request(&format!("/movies/{movie_id}/extended"), &[]).await;
        result.and_then(|v| v.get("data").cloned())
    }

    /// Get TV show details by ID.
    pub async fn get_tv_by_id(&self, tv_id: &str) -> Option<serde_json::Value> {
        let result: Option<serde_json::Value> =
            self.request(&format!("/series/{tv_id}/extended"), &[]).await;
        result.and_then(|v| v.get("data").cloned())
    }

    /// Get episode details.
    pub async fn get_episode(
        &self,
        tv_id: &str,
        season: u16,
        episode: u16,
    ) -> Option<serde_json::Value> {
        let result: Option<serde_json::Value> =
            self.request(&format!("/series/{tv_id}/episodes/default"), &[]).await;

        let episodes = result
            .and_then(|v| v.get("data").cloned())
            .and_then(|v| v.get("episodes").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();

        episodes.into_iter().find(|ep| {
            ep.get("seasonNumber").and_then(|s| s.as_u64()).map(|s| s as u16 == season)
                == Some(true)
                && ep.get("number").and_then(|n| n.as_u64()).map(|n| n as u16 == episode)
                    == Some(true)
        })
    }
}
