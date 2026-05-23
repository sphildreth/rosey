use camino::{Utf8Path, Utf8PathBuf};
use rosey_core::{ConflictPolicy, MediaItem, MoveResult};
use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("source does not exist: {0}")]
    SourceMissing(Utf8PathBuf),

    #[error("destination exists: {0}")]
    DestinationExists(Utf8PathBuf),

    #[error("filesystem operation failed: {0}")]
    Io(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRequest {
    pub source: Utf8PathBuf,
    pub destination: Utf8PathBuf,
    pub conflict_policy: ConflictPolicy,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveAction {
    Moved,
    Skipped,
    Replaced,
    KeptBoth,
    WouldMove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveOutcome {
    pub success: bool,
    pub action: MoveAction,
    pub source: Utf8PathBuf,
    pub destination: Utf8PathBuf,
    pub error: Option<String>,
}

/// Check whether source and destination are on the same filesystem volume.
///
/// Uses `st_dev` from `std::fs::metadata`. If either path cannot be queried,
/// returns `false` conservatively.
pub fn same_volume(source: &Utf8Path, dest: &Utf8Path) -> bool {
    let src_meta = match fs::metadata(source) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let dest_parent = match dest.parent() {
        Some(p) => p,
        None => return false,
    };

    let dest_meta = match fs::metadata(dest_parent) {
        Ok(m) => m,
        Err(_) => return false,
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        src_meta.dev() == dest_meta.dev()
    }

    #[cfg(not(unix))]
    {
        // On non-Unix platforms, conservatively return false.
        // Windows same-volume detection would need GetVolumeInformation.
        false
    }
}

/// Apply a `(1)`, `(2)`, etc. suffix to a destination path when the
/// `KeepBoth` conflict policy is active.
pub fn apply_conflict_suffix(dest_path: &Utf8Path) -> Utf8PathBuf {
    let parent = dest_path.parent().unwrap_or_else(|| Utf8Path::new(""));
    let stem = dest_path.file_stem().unwrap_or("");
    let ext = dest_path.extension().unwrap_or("");

    let mut counter = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}).{}", stem, counter, ext)
        };
        let candidate = parent.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

/// Move a single file with transactional guarantees.
///
/// * `dry_run` — report what would happen without touching files.
/// * Same volume — uses `std::fs::rename` (atomic).
/// * Cross volume — copy, verify size, then delete source.
/// * Conflict policies — `Skip`, `Replace`, `KeepBoth`.
pub fn move_file_transactional(
    source: &Utf8Path,
    dest: &Utf8Path,
    conflict_policy: ConflictPolicy,
    dry_run: bool,
) -> Result<(bool, MoveAction), MoveError> {
    if dry_run {
        return Ok((true, MoveAction::WouldMove));
    }

    if !source.exists() {
        return Err(MoveError::SourceMissing(source.to_path_buf()));
    }

    let mut action = MoveAction::Moved;
    let mut effective_dest = dest.to_path_buf();

    if dest.exists() {
        match conflict_policy {
            ConflictPolicy::Skip => {
                return Ok((true, MoveAction::Skipped));
            }
            ConflictPolicy::KeepBoth => {
                effective_dest = apply_conflict_suffix(dest);
                action = MoveAction::KeptBoth;
            }
            ConflictPolicy::Replace => {
                action = MoveAction::Replaced;
            }
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = effective_dest.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(MoveError::Io(format!("failed to create destination directory: {e}")));
        }
    }

    if same_volume(source, &effective_dest) {
        // Same volume: atomic rename
        if let Err(e) = fs::rename(source.as_std_path(), effective_dest.as_std_path()) {
            return Err(MoveError::Io(format!("rename failed: {e}")));
        }
    } else {
        // Cross volume: copy → verify → delete source
        if let Err(e) = fs::copy(source.as_std_path(), effective_dest.as_std_path()) {
            return Err(MoveError::Io(format!("copy failed: {e}")));
        }

        // Verify by size
        let src_size = fs::metadata(source).map(|m| m.len()).unwrap_or(u64::MAX);
        let dst_size = fs::metadata(&effective_dest).map(|m| m.len()).unwrap_or(0);

        if src_size != dst_size {
            let _ = fs::remove_file(effective_dest.as_std_path());
            return Err(MoveError::Io(
                "verification failed: destination size mismatch".to_string(),
            ));
        }

        // Verification passed — safe to remove source
        if let Err(e) = fs::remove_file(source.as_std_path()) {
            return Err(MoveError::Io(format!(
                "copied and verified but failed to remove source: {e}"
            )));
        }
    }

    Ok((true, action))
}

/// Preflight check results before moving files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreflightResult {
    pub free_space_ok: bool,
    pub path_len_ok: bool,
    pub perms_ok: bool,
    pub errors: Vec<String>,
}

/// Perform preflight checks before moving files.
pub fn check_preflight(sources: &[&Utf8Path], destination_dir: &Utf8Path) -> PreflightResult {
    let mut errors = Vec::new();

    // Ensure destination exists (create if needed)
    if !destination_dir.exists() {
        if let Err(e) = fs::create_dir_all(destination_dir) {
            return PreflightResult {
                free_space_ok: false,
                path_len_ok: false,
                perms_ok: false,
                errors: vec![format!("Cannot create destination: {e}")],
            };
        }
    }

    // Check writability by attempting a temporary file
    let probe = destination_dir.join(".rosey_write_probe");
    match fs::write(&probe, b"") {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
        }
        Err(e) => {
            errors.push(format!("Destination is not writable: {e}"));
        }
    }

    // Calculate total size
    let mut total_size: u64 = 0;
    for src in sources {
        if let Ok(meta) = fs::metadata(src) {
            total_size += meta.len();
        }
    }

    // Check free space (best-effort via fs2 or statvfs)
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        use std::os::unix::ffi::OsStrExt;

        let path_c = match CString::new(destination_dir.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => {
                errors.push("Invalid destination path for statvfs".to_string());
                return build_preflight(errors);
            }
        };

        let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
        let rc = unsafe { libc::statvfs(path_c.as_ptr(), stat.as_mut_ptr()) };
        if rc == 0 {
            let stat = unsafe { stat.assume_init() };
            let free_bytes = stat.f_bavail * stat.f_frsize;
            let buffer = 100 * 1024 * 1024; // 100 MiB buffer
            if free_bytes < total_size + buffer {
                errors.push(format!(
                    "Insufficient space: need {} bytes, have {}",
                    total_size + buffer,
                    free_bytes
                ));
            }
        }
    }

    // Check path length
    let dest_prefix_len = destination_dir.as_str().len() + 1; // +1 for separator
    for src in sources {
        if let Some(name) = src.file_name() {
            if dest_prefix_len + name.len() > 255 {
                errors.push(format!("Path too long: {src}"));
                break;
            }
        }
    }

    build_preflight(errors)
}

fn build_preflight(errors: Vec<String>) -> PreflightResult {
    let free_space_ok = !errors.iter().any(|e| e.contains("Insufficient space"));
    let path_len_ok = !errors.iter().any(|e| e.contains("Path too long"));
    let perms_ok = !errors.iter().any(|e| e.contains("not writable"));

    PreflightResult { free_space_ok, path_len_ok, perms_ok, errors }
}

/// Move a media item and its sidecars transactionally.
///
/// If any file fails to move, already-moved files are rolled back (deleted).
pub fn move_with_sidecars(
    item: &MediaItem,
    destination: &Utf8Path,
    conflict_policy: ConflictPolicy,
    dry_run: bool,
) -> MoveResult {
    let source = &item.source_path;
    let sidecars = crate::discover_sidecars(source);

    // Collect all sources
    let mut all_sources: Vec<Utf8PathBuf> = Vec::with_capacity(1 + sidecars.len());
    all_sources.push(source.clone());
    all_sources.extend(sidecars.clone());

    let mut moved_files: Vec<Utf8PathBuf> = Vec::new();
    let mut result = MoveResult {
        success: false,
        moved: Vec::new(),
        skipped: Vec::new(),
        replaced: Vec::new(),
        kept_both: Vec::new(),
        rollback_performed: false,
        errors: Vec::new(),
    };

    // Preflight
    let dest_dir = destination.parent().unwrap_or_else(|| Utf8Path::new(""));
    let preflight =
        check_preflight(&all_sources.iter().map(|p| p.as_ref()).collect::<Vec<_>>(), dest_dir);

    if !preflight.free_space_ok || !preflight.path_len_ok || !preflight.perms_ok {
        result.errors = preflight.errors;
        return result;
    }

    // Move main file
    match move_file_transactional(source, destination, conflict_policy, dry_run) {
        Ok((true, action)) => {
            record_action(&mut result, action, destination);
            moved_files.push(destination.to_path_buf());
        }
        Ok((false, _)) | Err(_) => {
            result.errors.push(format!("Failed to move {source}"));
            return result;
        }
    }

    // Move sidecars
    let dest_parent = destination.parent().unwrap_or_else(|| Utf8Path::new(""));
    let dest_stem = destination.file_stem().unwrap_or("");

    for sidecar in &sidecars {
        let sidecar_ext = sidecar.extension().unwrap_or("");
        let sidecar_dest = if sidecar_ext.is_empty() {
            dest_parent.join(dest_stem)
        } else {
            dest_parent.join(format!("{}.{}", dest_stem, sidecar_ext))
        };

        match move_file_transactional(sidecar, &sidecar_dest, conflict_policy, dry_run) {
            Ok((true, action)) => {
                record_action(&mut result, action, &sidecar_dest);
                moved_files.push(sidecar_dest);
            }
            Ok((false, _)) | Err(_) => {
                // Rollback
                if !dry_run {
                    for moved in &moved_files {
                        let _ = fs::remove_file(moved.as_std_path());
                    }
                    result.rollback_performed = true;
                }
                result.errors.push(format!("Failed to move sidecar {sidecar}, rolled back"));
                return result;
            }
        }
    }

    result.success = true;
    result
}

fn record_action(result: &mut MoveResult, action: MoveAction, path: &Utf8Path) {
    match action {
        MoveAction::Moved => result.moved.push(path.to_path_buf()),
        MoveAction::Skipped => result.skipped.push(path.to_path_buf()),
        MoveAction::Replaced => result.replaced.push(path.to_path_buf()),
        MoveAction::KeptBoth => result.kept_both.push(path.to_path_buf()),
        MoveAction::WouldMove => {}
    }
}
