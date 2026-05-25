use crate::RoseyConfig;
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
};
use tracing_subscriber::EnvFilter;

pub type LoggingInitResult = Result<(), Box<dyn Error + Send + Sync>>;

pub fn init_tracing(config: &RoseyConfig) -> LoggingInitResult {
    let filter = logging_filter(config);
    let file_path = config.logging.file_path.trim();

    if file_path.is_empty() {
        if config.logging.log_to_console {
            tracing_subscriber::fmt().with_env_filter(filter).try_init()?;
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_ansi(false)
                .with_writer(io::sink)
                .try_init()?;
        }
        return Ok(());
    }

    let path = PathBuf::from(file_path);
    prepare_log_file(&path, config)?;
    let file = OpenOptions::new().create(true).append(true).open(&path)?;

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(move || file.try_clone().expect("failed to clone Rosey log file handle"))
        .try_init()?;

    Ok(())
}

pub fn init_fallback_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("warn"))
        .with_ansi(false)
        .with_writer(io::sink)
        .try_init();
}

fn logging_filter(config: &RoseyConfig) -> EnvFilter {
    let directive = std::env::var("ROSEY_LOG")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| config.logging.level.to_ascii_lowercase());

    EnvFilter::try_new(&directive).unwrap_or_else(|_| EnvFilter::new("info"))
}

fn prepare_log_file(path: &Path, config: &RoseyConfig) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }

    rotate_log_file_if_needed(
        path,
        u64::from(config.logging.max_file_size_mb.max(1)) * 1024 * 1024,
        config.logging.backup_count,
    )
}

fn rotate_log_file_if_needed(path: &Path, max_bytes: u64, backup_count: u32) -> io::Result<()> {
    let Ok(metadata) = fs::metadata(path) else {
        return Ok(());
    };
    if metadata.len() <= max_bytes {
        return Ok(());
    }

    if backup_count == 0 {
        fs::remove_file(path)?;
        return Ok(());
    }

    for index in (1..=backup_count).rev() {
        let rotated = rotated_log_path(path, index);
        if !rotated.exists() {
            continue;
        }

        if index == backup_count {
            fs::remove_file(rotated)?;
        } else {
            fs::rename(rotated, rotated_log_path(path, index + 1))?;
        }
    }

    fs::rename(path, rotated_log_path(path, 1))?;
    Ok(())
}

fn rotated_log_path(path: &Path, index: u32) -> PathBuf {
    let mut rotated = path.to_path_buf();
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("rosey.log");
    rotated.set_file_name(format!("{file_name}.{index}"));
    rotated
}
