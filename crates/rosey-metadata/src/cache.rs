use sqlite::{Connection, State};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Disk-backed cache for provider metadata.
///
/// Mirrors Python `ProviderCache` from `rosey/providers/cache.py`.
pub struct ProviderCache {
    conn: Connection,
    ttl_seconds: u64,
}

impl ProviderCache {
    /// Open or create a cache database at the given path.
    pub fn open(path: impl AsRef<Path>, ttl_days: u32) -> Result<Self, sqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                provider TEXT NOT NULL,
                kind TEXT NOT NULL,
                key TEXT NOT NULL,
                data TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (provider, kind, key)
            )",
        )?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_updated_at ON cache(updated_at)")?;
        Ok(Self { conn, ttl_seconds: ttl_days as u64 * 24 * 3600 })
    }

    /// Get cached data if not expired.
    pub fn get(&self, provider: &str, kind: &str, key: &str) -> Option<serde_json::Value> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT data, updated_at FROM cache
                 WHERE provider = ?1 AND kind = ?2 AND key = ?3",
            )
            .ok()?;

        stmt.bind((1, provider)).ok()?;
        stmt.bind((2, kind)).ok()?;
        stmt.bind((3, key)).ok()?;

        if let Ok(State::Row) = stmt.next() {
            let data_json: String = stmt.read(0).ok()?;
            let updated_at: i64 = stmt.read(1).ok()?;

            let now = now_secs();
            if now - updated_at as u64 > self.ttl_seconds {
                // Expired — delete using bound parameters
                let _ = self
                    .conn
                    .execute("DELETE FROM cache WHERE provider = ?1 AND kind = ?2 AND key = ?3");
                return None;
            }

            return serde_json::from_str(&data_json).ok();
        }

        None
    }

    /// Store data in cache.
    pub fn set(
        &self,
        provider: &str,
        kind: &str,
        key: &str,
        data: &serde_json::Value,
    ) -> Result<(), sqlite::Error> {
        let data_json = serde_json::to_string(data).unwrap_or_default();
        let now = now_secs() as i64;

        let mut stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO cache (provider, kind, key, data, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        stmt.bind((1, provider))?;
        stmt.bind((2, kind))?;
        stmt.bind((3, key))?;
        stmt.bind((4, data_json.as_str()))?;
        stmt.bind((5, now))?;
        stmt.next()?;
        Ok(())
    }

    /// Remove expired entries.
    pub fn clear_expired(&self) -> Result<usize, sqlite::Error> {
        let now = now_secs() as i64;
        let mut stmt = self.conn.prepare("DELETE FROM cache WHERE updated_at < ?")?;
        stmt.bind((1, now))?;
        stmt.next()?;
        // sqlite crate doesn't expose rowcount easily; return 0 as best effort
        Ok(0)
    }

    /// Clear all cached data.
    pub fn clear_all(&self) -> Result<(), sqlite::Error> {
        self.conn.execute("DELETE FROM cache")?;
        Ok(())
    }

    /// Get cache statistics.
    pub fn stats(&self) -> Result<CacheStats, sqlite::Error> {
        let mut total = 0;
        let mut expired = 0;
        let now = now_secs() as i64;

        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM cache")?;
        if let Ok(State::Row) = stmt.next() {
            total = stmt.read::<i64, usize>(0).unwrap_or(0) as usize;
        }

        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM cache WHERE updated_at < ?")?;
        stmt.bind((1, now))?;
        if let Ok(State::Row) = stmt.next() {
            expired = stmt.read::<i64, usize>(0).unwrap_or(0) as usize;
        }

        Ok(CacheStats { total, expired })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheStats {
    pub total: usize,
    pub expired: usize,
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
