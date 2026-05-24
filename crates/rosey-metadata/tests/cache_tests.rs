use rosey_metadata::ProviderCache;

fn temp_db() -> (tempfile::TempDir, String) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("cache.db");
    (tmp, path.to_str().unwrap().to_string())
}

#[test]
fn cache_open_creates_schema() {
    let (_tmp, path) = temp_db();
    let cache = ProviderCache::open(&path, 30).unwrap();
    let stats = cache.stats().unwrap();
    assert_eq!(stats.total, 0);
}

#[test]
fn cache_open_accepts_python_style_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_dir = tmp.path().join("cache");

    let cache = ProviderCache::open(&cache_dir, 30).unwrap();

    assert!(cache_dir.join("provider_cache.db").exists());
    assert_eq!(cache.stats().unwrap().total, 0);
}

#[test]
fn cache_set_and_get() {
    let (_tmp, path) = temp_db();
    let cache = ProviderCache::open(&path, 30).unwrap();

    let data = serde_json::json!({"title": "The Matrix", "year": 1999});
    cache.set("tmdb", "movie", "603", &data).unwrap();

    let cached = cache.get("tmdb", "movie", "603");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap()["title"], "The Matrix");
}

#[test]
fn cache_get_missing_returns_none() {
    let (_tmp, path) = temp_db();
    let cache = ProviderCache::open(&path, 30).unwrap();
    assert!(cache.get("tmdb", "movie", "99999").is_none());
}

#[test]
fn cache_expired_entry_returns_none() {
    let (_tmp, path) = temp_db();
    let cache = ProviderCache::open(&path, 0).unwrap(); // 0-day TTL = immediate expiry

    let data = serde_json::json!({"title": "Old"});
    cache.set("tmdb", "movie", "1", &data).unwrap();

    // Sleep briefly to ensure the entry is past TTL
    std::thread::sleep(std::time::Duration::from_millis(1100));

    // Should be expired
    assert!(cache.get("tmdb", "movie", "1").is_none());
    assert_eq!(cache.stats().unwrap().total, 0);
}

#[test]
fn cache_clear_expired_returns_removed_count() {
    let (_tmp, path) = temp_db();
    let cache = ProviderCache::open(&path, 0).unwrap();

    let data = serde_json::json!({"title": "Old"});
    cache.set("tmdb", "movie", "1", &data).unwrap();
    cache.set("tmdb", "movie", "2", &data).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1100));

    let removed = cache.clear_expired().unwrap();

    assert_eq!(removed, 2);
    assert_eq!(cache.stats().unwrap().total, 0);
}

#[test]
fn cache_clear_all() {
    let (_tmp, path) = temp_db();
    let cache = ProviderCache::open(&path, 30).unwrap();

    let data = serde_json::json!({"title": "Test"});
    cache.set("tmdb", "movie", "1", &data).unwrap();
    cache.clear_all().unwrap();

    assert!(cache.get("tmdb", "movie", "1").is_none());
    let stats = cache.stats().unwrap();
    assert_eq!(stats.total, 0);
}

#[test]
fn cache_stats_counts_entries() {
    let (_tmp, path) = temp_db();
    let cache = ProviderCache::open(&path, 30).unwrap();

    let data = serde_json::json!({"title": "A"});
    cache.set("tmdb", "movie", "1", &data).unwrap();
    cache.set("tmdb", "movie", "2", &data).unwrap();

    let stats = cache.stats().unwrap();
    assert_eq!(stats.total, 2);
}
