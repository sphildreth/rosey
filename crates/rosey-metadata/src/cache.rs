use decentdb::{Db, DbConfig, DbError, QueryResult, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const DECENTDB_MIGRATE_BIN_ENV: &str = "ROSEY_DECENTDB_MIGRATE";
const DECENTDB_RESET_ON_UNSUPPORTED_ENV: &str = "ROSEY_DECENTDB_RESET_ON_UNSUPPORTED";

/// Disk-backed cache for provider metadata.
///
/// Mirrors Python `ProviderCache` from `rosey/providers/cache.py`.
pub struct ProviderCache {
    db: Db,
    ttl_seconds: u64,
}

impl ProviderCache {
    /// Open or create a cache database at the given path.
    pub fn open(path: impl AsRef<Path>, ttl_days: u32) -> decentdb::Result<Self> {
        let db_path = cache_db_path(path.as_ref());
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| DbError::io("failed to create cache directory", err))?;
        }

        let db = open_or_update_db(&db_path)?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                provider TEXT NOT NULL,
                kind TEXT NOT NULL,
                key TEXT NOT NULL,
                data TEXT NOT NULL,
                updated_at INT64 NOT NULL,
                PRIMARY KEY (provider, kind, key)
            )",
        )?;
        db.execute("CREATE INDEX IF NOT EXISTS idx_updated_at ON cache(updated_at)")?;
        Ok(Self { db, ttl_seconds: ttl_days as u64 * 24 * 3600 })
    }

    /// Get cached data if not expired.
    pub fn get(&self, provider: &str, kind: &str, key: &str) -> Option<serde_json::Value> {
        let result = self
            .db
            .execute_with_params(
                "SELECT data, updated_at FROM cache
                 WHERE provider = $1 AND kind = $2 AND key = $3",
                &[
                    Value::Text(provider.to_string()),
                    Value::Text(kind.to_string()),
                    Value::Text(key.to_string()),
                ],
            )
            .ok()?;
        let values = result.rows().first()?.values();
        let data_json = text_value(values.first()?)?;
        let updated_at = int_value(values.get(1)?)?;

        if updated_at < self.expiry_cutoff() {
            let _ = self.delete_key(provider, kind, key);
            return None;
        }

        serde_json::from_str(data_json).ok()
    }

    /// Store data in cache.
    pub fn set(
        &self,
        provider: &str,
        kind: &str,
        key: &str,
        data: &serde_json::Value,
    ) -> decentdb::Result<()> {
        let data_json = serde_json::to_string(data).unwrap_or_default();
        let now = now_secs() as i64;

        self.db.execute_with_params(
            "INSERT INTO cache (provider, kind, key, data, updated_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (provider, kind, key) DO UPDATE
             SET data = EXCLUDED.data,
                 updated_at = EXCLUDED.updated_at",
            &[
                Value::Text(provider.to_string()),
                Value::Text(kind.to_string()),
                Value::Text(key.to_string()),
                Value::Text(data_json),
                Value::Int64(now),
            ],
        )?;
        Ok(())
    }

    /// Remove expired entries.
    pub fn clear_expired(&self) -> decentdb::Result<usize> {
        let cutoff = self.expiry_cutoff();

        let count = self.db.execute_with_params(
            "SELECT COUNT(*) FROM cache WHERE updated_at < $1",
            &[Value::Int64(cutoff)],
        )?;
        let expired = first_int(&count).unwrap_or(0) as usize;

        self.db.execute_with_params(
            "DELETE FROM cache WHERE updated_at < $1",
            &[Value::Int64(cutoff)],
        )?;
        Ok(expired)
    }

    /// Clear all cached data.
    pub fn clear_all(&self) -> decentdb::Result<()> {
        self.db.execute("DELETE FROM cache")?;
        Ok(())
    }

    /// Get cache statistics.
    pub fn stats(&self) -> decentdb::Result<CacheStats> {
        let cutoff = self.expiry_cutoff();

        let total_result = self.db.execute("SELECT COUNT(*) FROM cache")?;
        let total = first_int(&total_result).unwrap_or(0) as usize;
        let expired_result = self.db.execute_with_params(
            "SELECT COUNT(*) FROM cache WHERE updated_at < $1",
            &[Value::Int64(cutoff)],
        )?;
        let expired = first_int(&expired_result).unwrap_or(0) as usize;

        Ok(CacheStats { total, expired })
    }

    fn delete_key(&self, provider: &str, kind: &str, key: &str) -> decentdb::Result<()> {
        self.db.execute_with_params(
            "DELETE FROM cache WHERE provider = $1 AND kind = $2 AND key = $3",
            &[
                Value::Text(provider.to_string()),
                Value::Text(kind.to_string()),
                Value::Text(key.to_string()),
            ],
        )?;
        Ok(())
    }

    fn expiry_cutoff(&self) -> i64 {
        now_secs().saturating_sub(self.ttl_seconds) as i64
    }
}

fn cache_db_path(path: &Path) -> PathBuf {
    if path.extension().is_some() {
        path.to_path_buf()
    } else {
        path.join("provider_cache.ddb")
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

fn open_or_update_db(path: &Path) -> decentdb::Result<Db> {
    match Db::open_or_create(path, DbConfig::default()) {
        Ok(db) => Ok(db),
        Err(err @ DbError::UnsupportedFormatVersion { .. }) => {
            if reset_on_unsupported() {
                std::fs::remove_file(path).map_err(|source| {
                    DbError::io("failed to remove unsupported cache database", source)
                })?;
                return Db::open_or_create(path, DbConfig::default());
            }

            migrate_decentdb_file(path, err)?;
            Db::open(path, DbConfig::default())
        }
        Err(err) => Err(err),
    }
}

fn migrate_decentdb_file(path: &Path, original_error: DbError) -> decentdb::Result<()> {
    let migrating_path = suffixed_path(path, ".migrating");
    let backup_path = suffixed_path(path, ".backup");

    remove_file_if_exists(&migrating_path)?;
    remove_file_if_exists(&backup_path)?;

    let output = Command::new(decentdb_migrate_bin())
        .arg("--source")
        .arg(path)
        .arg("--dest")
        .arg(&migrating_path)
        .output()
        .map_err(|source| DbError::io("failed to run decentdb-migrate", source))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(DbError::internal(format!(
            "decentdb-migrate failed after {original_error}: {stderr}{stdout}"
        )));
    }

    Db::open(&migrating_path, DbConfig::default())?;

    std::fs::rename(path, &backup_path)
        .map_err(|source| DbError::io("failed to back up unsupported cache database", source))?;
    if let Err(source) = std::fs::rename(&migrating_path, path) {
        let restore_result = std::fs::rename(&backup_path, path);
        let restore_detail = restore_result
            .err()
            .map(|err| format!("; failed to restore backup: {err}"))
            .unwrap_or_default();
        return Err(DbError::io(
            format!("failed to install migrated cache database{restore_detail}"),
            source,
        ));
    }

    Ok(())
}

fn remove_file_if_exists(path: &Path) -> decentdb::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(DbError::io("failed to remove stale DecentDB migration file", err)),
    }
}

fn suffixed_path(path: &Path, suffix: &str) -> PathBuf {
    let mut path = path.as_os_str().to_os_string();
    path.push(suffix);
    PathBuf::from(path)
}

fn decentdb_migrate_bin() -> String {
    std::env::var(DECENTDB_MIGRATE_BIN_ENV).unwrap_or_else(|_| "decentdb-migrate".to_string())
}

fn reset_on_unsupported() -> bool {
    std::env::var(DECENTDB_RESET_ON_UNSUPPORTED_ENV)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn first_int(result: &QueryResult) -> Option<i64> {
    int_value(result.rows().first()?.values().first()?)
}

fn int_value(value: &Value) -> Option<i64> {
    match value {
        Value::Int64(value) => Some(*value),
        _ => None,
    }
}

fn text_value(value: &Value) -> Option<&str> {
    match value {
        Value::Text(value) => Some(value.as_str()),
        _ => None,
    }
}
