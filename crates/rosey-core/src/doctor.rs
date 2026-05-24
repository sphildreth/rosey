use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{config_path, RoseyConfig};

const DECENTDB_MIGRATE_BIN_ENV: &str = "ROSEY_DECENTDB_MIGRATE";
const DECENTDB_RESET_ON_UNSUPPORTED_ENV: &str = "ROSEY_DECENTDB_RESET_ON_UNSUPPORTED";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DoctorStatus {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: DoctorStatus,
    pub message: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub overall: DoctorStatus,
    pub config_path: String,
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    #[must_use]
    pub fn errors(&self) -> usize {
        self.checks.iter().filter(|check| check.status == DoctorStatus::Error).count()
    }

    #[must_use]
    pub fn warnings(&self) -> usize {
        self.checks.iter().filter(|check| check.status == DoctorStatus::Warn).count()
    }
}

#[must_use]
pub fn run_doctor(config: &RoseyConfig) -> DoctorReport {
    let config_path = config_path();
    let mut checks = Vec::new();

    check_config_path(&config_path, &mut checks);
    check_source_path(&config.paths.source, &mut checks);
    check_target_path("Movies target", &config.paths.movies, &mut checks);
    check_target_path("TV target", &config.paths.tv, &mut checks);
    check_cache_path(&config_path, &mut checks);
    check_log_path(&config.logging.file_path, &mut checks);
    check_confidence_thresholds(config, &mut checks);
    check_worker_counts(config, &mut checks);
    check_conflict_policy(&config.behavior.conflict_policy, &mut checks);
    check_provider_config(config, &mut checks);
    check_decentdb_migrate(&mut checks);

    let overall = if checks.iter().any(|check| check.status == DoctorStatus::Error) {
        DoctorStatus::Error
    } else if checks.iter().any(|check| check.status == DoctorStatus::Warn) {
        DoctorStatus::Warn
    } else {
        DoctorStatus::Ok
    };

    DoctorReport { overall, config_path: config_path.display().to_string(), checks }
}

fn check_config_path(path: &Path, checks: &mut Vec<DoctorCheck>) {
    if let Some(parent) = path.parent() {
        check_existing_dir("Config directory", parent, true, true, checks);
    } else {
        checks.push(error(
            "Config path",
            "Config path has no parent directory.",
            path_detail(path),
        ));
    }

    if path.exists() {
        if path.is_file() {
            match std::fs::File::open(path) {
                Ok(_) => {
                    checks.push(ok("Config file", "Config file is readable.", path_detail(path)))
                }
                Err(err) => checks.push(error_detail(
                    "Config file",
                    "Config file exists but cannot be read.",
                    path_detail(path),
                    err,
                )),
            }
        } else {
            checks.push(error(
                "Config file",
                "Config path exists but is not a file.",
                path_detail(path),
            ));
        }
    } else {
        checks.push(warn(
            "Config file",
            "Config file does not exist yet; defaults are active.",
            path_detail(path),
        ));
    }
}

fn check_source_path(value: &str, checks: &mut Vec<DoctorCheck>) {
    if value.trim().is_empty() {
        checks.push(error("Source path", "No source path is configured.", ""));
        return;
    }

    let path = Path::new(value);
    if !path.exists() {
        checks.push(error("Source path", "Source path does not exist.", path_detail(path)));
        return;
    }

    check_existing_dir("Source path", path, true, false, checks);
}

fn check_target_path(name: &str, value: &str, checks: &mut Vec<DoctorCheck>) {
    if value.trim().is_empty() {
        checks.push(warn(name, "Target path is not configured.", ""));
        return;
    }

    let path = Path::new(value);
    if path.exists() {
        check_existing_dir(name, path, true, true, checks);
        return;
    }

    match path.parent() {
        Some(parent) if parent.exists() => {
            if probe_write(parent).is_ok() {
                checks.push(warn(
                    name,
                    "Target path does not exist, but its parent is writable.",
                    path_detail(path),
                ));
            } else {
                checks.push(error(
                    name,
                    "Target path does not exist and parent is not writable.",
                    path_detail(path),
                ));
            }
        }
        _ => checks.push(error(
            name,
            "Target path does not exist and has no existing parent.",
            path_detail(path),
        )),
    }
}

fn check_cache_path(config_path: &Path, checks: &mut Vec<DoctorCheck>) {
    let Some(config_dir) = config_path.parent() else {
        checks.push(error(
            "Provider cache",
            "Config path has no cache parent directory.",
            path_detail(config_path),
        ));
        return;
    };
    let cache_dir = config_dir.join("cache");

    if cache_dir.exists() {
        check_existing_dir("Provider cache", &cache_dir, true, true, checks);
    } else if probe_write(config_dir).is_ok() {
        checks.push(ok(
            "Provider cache",
            "Cache directory does not exist yet, but config directory is writable.",
            path_detail(&cache_dir),
        ));
    } else {
        checks.push(error(
            "Provider cache",
            "Cache directory does not exist and config directory is not writable.",
            path_detail(&cache_dir),
        ));
    }
}

fn check_log_path(value: &str, checks: &mut Vec<DoctorCheck>) {
    if value.trim().is_empty() {
        checks.push(ok("Log file", "File logging is disabled.", ""));
        return;
    }

    let path = Path::new(value);
    if path.exists() && !path.is_file() {
        checks.push(error(
            "Log file",
            "Configured log path exists but is not a file.",
            path_detail(path),
        ));
        return;
    }

    let Some(parent) = path.parent() else {
        checks.push(error(
            "Log file",
            "Configured log file has no parent directory.",
            path_detail(path),
        ));
        return;
    };

    if parent.exists() && probe_write(parent).is_ok() {
        checks.push(ok("Log file", "Log file parent is writable.", path_detail(path)));
    } else {
        checks.push(error(
            "Log file",
            "Log file parent is missing or not writable.",
            path_detail(path),
        ));
    }
}

fn check_confidence_thresholds(config: &RoseyConfig, checks: &mut Vec<DoctorCheck>) {
    let green = config.identification.confidence_thresholds.green;
    let yellow = config.identification.confidence_thresholds.yellow;

    if green > 100 || yellow > 100 {
        checks.push(error(
            "Confidence thresholds",
            "Confidence thresholds must be between 0 and 100.",
            format!("green={green}, yellow={yellow}"),
        ));
    } else if green < yellow {
        checks.push(warn(
            "Confidence thresholds",
            "Green threshold is below yellow threshold; confidence bands will be inverted.",
            format!("green={green}, yellow={yellow}"),
        ));
    } else {
        checks.push(ok(
            "Confidence thresholds",
            "Confidence thresholds are in the expected order.",
            format!("green={green}, yellow={yellow}"),
        ));
    }
}

fn check_worker_counts(config: &RoseyConfig, checks: &mut Vec<DoctorCheck>) {
    let local = config.scanning.concurrency_local;
    let network = config.scanning.concurrency_network;
    let cores = std::thread::available_parallelism().map(usize::from).unwrap_or(1);
    let high_local = cores.saturating_mul(4).max(8);

    if local == 0 {
        checks.push(error("Local workers", "Local worker count must be at least 1.", local));
    } else if local > high_local {
        checks.push(warn(
            "Local workers",
            "Local worker count is unusually high for this system.",
            format!("configured={local}, logical_cpus={cores}"),
        ));
    } else {
        checks.push(ok("Local workers", "Local worker count is reasonable.", local));
    }

    if network == 0 {
        checks.push(warn("Network workers", "Network worker count is zero.", network));
    } else {
        checks.push(ok("Network workers", "Network worker count is configured.", network));
    }
}

fn check_conflict_policy(policy: &str, checks: &mut Vec<DoctorCheck>) {
    match policy {
        "ask" | "skip" | "replace" | "keep_both" => {
            checks.push(ok("Conflict policy", "Conflict policy is valid.", policy))
        }
        _ => checks.push(error(
            "Conflict policy",
            "Conflict policy must be ask, skip, replace, or keep_both.",
            policy,
        )),
    }
}

fn check_provider_config(config: &RoseyConfig, checks: &mut Vec<DoctorCheck>) {
    if config.identification.use_online_providers {
        if config.providers.tmdb_api_key.trim().is_empty() {
            checks.push(error(
                "TMDB provider",
                "Online providers are enabled but TMDB API key is missing.",
                "providers.tmdb_api_key",
            ));
        } else {
            checks.push(ok(
                "TMDB provider",
                "TMDB API key is configured.",
                "providers.tmdb_api_key",
            ));
        }

        if config.providers.tvdb_api_key.trim().is_empty() {
            checks.push(warn(
                "TVDB provider",
                "TVDB API key is missing; TMDB-only lookups can still work.",
                "providers.tvdb_api_key",
            ));
        } else {
            checks.push(ok(
                "TVDB provider",
                "TVDB API key is configured.",
                "providers.tvdb_api_key",
            ));
        }
    } else {
        checks.push(ok("Online providers", "Online provider lookups are disabled.", ""));
    }

    if config.providers.cache_ttl_days == 0 {
        checks.push(warn(
            "Provider cache TTL",
            "Provider cache TTL is zero; entries expire immediately.",
            0,
        ));
    } else {
        checks.push(ok(
            "Provider cache TTL",
            "Provider cache TTL is configured.",
            format!("{} days", config.providers.cache_ttl_days),
        ));
    }
}

fn check_decentdb_migrate(checks: &mut Vec<DoctorCheck>) {
    if reset_on_unsupported() {
        checks.push(warn(
            "DecentDB migrate",
            "Unsupported DecentDB cache files will be deleted instead of migrated.",
            DECENTDB_RESET_ON_UNSUPPORTED_ENV,
        ));
        return;
    }

    let configured = std::env::var(DECENTDB_MIGRATE_BIN_ENV).ok();
    let candidate = configured.as_deref().unwrap_or("decentdb-migrate");

    if find_executable(candidate).is_some() {
        checks.push(ok("DecentDB migrate", "decentdb-migrate is available.", candidate));
    } else if configured.is_some() {
        checks.push(error(
            "DecentDB migrate",
            "ROSEY_DECENTDB_MIGRATE points to a missing or non-executable file.",
            candidate,
        ));
    } else {
        checks.push(warn(
            "DecentDB migrate",
            "decentdb-migrate was not found in PATH; old DecentDB cache formats cannot be updated automatically.",
            candidate,
        ));
    }
}

fn check_existing_dir(
    name: &str,
    path: &Path,
    require_read: bool,
    require_write: bool,
    checks: &mut Vec<DoctorCheck>,
) {
    if !path.is_dir() {
        checks.push(error(name, "Path exists but is not a directory.", path_detail(path)));
        return;
    }

    if require_read && std::fs::read_dir(path).is_err() {
        checks.push(error(name, "Directory is not readable.", path_detail(path)));
        return;
    }

    if require_write && probe_write(path).is_err() {
        checks.push(error(name, "Directory is not writable.", path_detail(path)));
        return;
    }

    checks.push(ok(name, "Directory is accessible.", path_detail(path)));
}

fn probe_write(dir: &Path) -> std::io::Result<()> {
    let probe = dir.join(format!(".rosey-doctor-{}", std::process::id()));
    std::fs::write(&probe, b"")?;
    std::fs::remove_file(probe)
}

fn find_executable(command: &str) -> Option<PathBuf> {
    let path = Path::new(command);
    if path.components().count() > 1 {
        return path.is_file().then(|| path.to_path_buf());
    }

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return Some(candidate);
        }

        #[cfg(windows)]
        {
            let candidate = dir.join(format!("{command}.exe"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

fn reset_on_unsupported() -> bool {
    std::env::var(DECENTDB_RESET_ON_UNSUPPORTED_ENV)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn path_detail(path: impl AsRef<Path>) -> String {
    path.as_ref().display().to_string()
}

fn ok(name: impl Into<String>, message: impl Into<String>, detail: impl ToString) -> DoctorCheck {
    check(name, DoctorStatus::Ok, message, detail)
}

fn warn(name: impl Into<String>, message: impl Into<String>, detail: impl ToString) -> DoctorCheck {
    check(name, DoctorStatus::Warn, message, detail)
}

fn error(
    name: impl Into<String>,
    message: impl Into<String>,
    detail: impl ToString,
) -> DoctorCheck {
    check(name, DoctorStatus::Error, message, detail)
}

fn error_detail(
    name: impl Into<String>,
    message: impl Into<String>,
    detail: impl ToString,
    err: impl ToString,
) -> DoctorCheck {
    DoctorCheck {
        name: name.into(),
        status: DoctorStatus::Error,
        message: message.into(),
        detail: Some(format!("{}: {}", detail.to_string(), err.to_string())),
    }
}

fn check(
    name: impl Into<String>,
    status: DoctorStatus,
    message: impl Into<String>,
    detail: impl ToString,
) -> DoctorCheck {
    let detail = detail.to_string();
    DoctorCheck {
        name: name.into(),
        status,
        message: message.into(),
        detail: (!detail.is_empty()).then_some(detail),
    }
}
