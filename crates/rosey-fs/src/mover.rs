use camino::{Utf8Path, Utf8PathBuf};
use rosey_core::{ConflictPolicy, MediaItem, MoveResult, ShowAssetKind, TvShow};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use thiserror::Error;

use crate::journal::{JournalOp, OperationJournal};
use crate::sidecars::is_sidecar_path;

const VERIFY_CHUNK_SIZE: usize = 1024 * 1024;
const VERIFY_CONTENT_THRESHOLD: u64 = 1024 * 1024;

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
        false
    }
}

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

pub fn move_file_transactional(
    source: &Utf8Path,
    dest: &Utf8Path,
    conflict_policy: ConflictPolicy,
    dry_run: bool,
) -> Result<(bool, MoveAction), MoveError> {
    move_file_transactional_journaled(source, dest, conflict_policy, dry_run, None)
}

pub fn move_file_transactional_journaled(
    source: &Utf8Path,
    dest: &Utf8Path,
    conflict_policy: ConflictPolicy,
    dry_run: bool,
    journal: Option<&OperationJournal>,
) -> Result<(bool, MoveAction), MoveError> {
    tracing::debug!(
        source = %source,
        destination = %dest,
        conflict_policy = ?conflict_policy,
        dry_run,
        journaled = journal.is_some(),
        "transactional move started"
    );

    if dry_run {
        tracing::debug!(source = %source, destination = %dest, "transactional move dry-run");
        return Ok((true, MoveAction::WouldMove));
    }

    if !source.exists() {
        tracing::debug!(source = %source, destination = %dest, "transactional move source missing");
        if let Some(j) = journal {
            j.record_error(JournalOp::Failed, source, dest, "source does not exist");
        }
        return Err(MoveError::SourceMissing(source.to_path_buf()));
    }

    let src_size = fs::metadata(source).map(|m| m.len()).unwrap_or(0);

    let mut action = MoveAction::Moved;
    let mut effective_dest = dest.to_path_buf();

    if dest.exists() {
        match conflict_policy {
            ConflictPolicy::Skip => {
                tracing::debug!(
                    source = %source,
                    destination = %dest,
                    "transactional move skipped because destination exists"
                );
                if let Some(j) = journal {
                    j.record_op(JournalOp::Skipped, source, dest);
                }
                return Ok((true, MoveAction::Skipped));
            }
            ConflictPolicy::KeepBoth => {
                effective_dest = apply_conflict_suffix(dest);
                action = MoveAction::KeptBoth;
                tracing::debug!(
                    source = %source,
                    requested_destination = %dest,
                    effective_destination = %effective_dest,
                    "transactional move keeping both files"
                );
                if let Some(j) = journal {
                    j.record_op(JournalOp::KeptBoth, source, &effective_dest);
                }
            }
            ConflictPolicy::Replace => {
                action = MoveAction::Replaced;
                tracing::debug!(
                    source = %source,
                    destination = %dest,
                    "transactional move replacing existing destination"
                );
                if let Some(j) = journal {
                    j.record_op(JournalOp::Replaced, source, dest);
                }
            }
        }
    }

    if let Some(parent) = effective_dest.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            tracing::debug!(
                source = %source,
                destination = %effective_dest,
                parent = %parent,
                error = %e,
                "transactional move failed to create destination directory"
            );
            if let Some(j) = journal {
                j.record_error(JournalOp::Failed, source, &effective_dest, &e.to_string());
            }
            return Err(MoveError::Io(format!("failed to create destination directory: {e}")));
        }
    }

    let same_volume = same_volume(source, &effective_dest);
    tracing::debug!(
        source = %source,
        destination = %effective_dest,
        same_volume,
        source_size_bytes = src_size,
        "transactional move path selected"
    );

    if same_volume {
        if let Some(j) = journal {
            j.record_op(JournalOp::MoveStarted, source, &effective_dest);
        }

        match fs::rename(source.as_std_path(), effective_dest.as_std_path()) {
            Ok(_) => {
                tracing::debug!(
                    source = %source,
                    destination = %effective_dest,
                    action = ?action,
                    "transactional rename complete"
                );
                if let Some(j) = journal {
                    j.record_op(JournalOp::Completed, source, &effective_dest);
                }
            }
            Err(e) => {
                tracing::debug!(
                    source = %source,
                    destination = %effective_dest,
                    error = %e,
                    "transactional rename failed"
                );
                if let Some(j) = journal {
                    j.record_error(JournalOp::Failed, source, &effective_dest, &e.to_string());
                }
                return Err(MoveError::Io(format!("rename failed: {e}")));
            }
        }
    } else {
        if let Some(j) = journal {
            j.record(
                &crate::journal::JournalEntry::now(JournalOp::CopyStarted, source, &effective_dest)
                    .with_bytes(src_size),
            );
        }

        if let Err(e) = fs::copy(source.as_std_path(), effective_dest.as_std_path()) {
            tracing::debug!(
                source = %source,
                destination = %effective_dest,
                error = %e,
                "transactional copy failed"
            );
            if let Some(j) = journal {
                j.record_error(JournalOp::Failed, source, &effective_dest, &e.to_string());
            }
            return Err(MoveError::Io(format!("copy failed: {e}")));
        }

        let dst_size = fs::metadata(&effective_dest).map(|m| m.len()).unwrap_or(0);
        if src_size != dst_size {
            tracing::debug!(
                source = %source,
                destination = %effective_dest,
                source_size_bytes = src_size,
                destination_size_bytes = dst_size,
                "transactional copy verification failed size check"
            );
            let _ = fs::remove_file(effective_dest.as_std_path());
            if let Some(j) = journal {
                j.record_error(
                    JournalOp::Failed,
                    source,
                    &effective_dest,
                    "verification failed: destination size mismatch",
                );
            }
            return Err(MoveError::Io(
                "verification failed: destination size mismatch".to_string(),
            ));
        }

        match verify_file_copy(source, &effective_dest) {
            Ok(true) => {
                tracing::debug!(
                    source = %source,
                    destination = %effective_dest,
                    "transactional copy verification passed"
                );
            }
            Ok(false) => {
                tracing::debug!(
                    source = %source,
                    destination = %effective_dest,
                    "transactional copy verification failed content check"
                );
                let _ = fs::remove_file(effective_dest.as_std_path());
                if let Some(j) = journal {
                    j.record_error(
                        JournalOp::Failed,
                        source,
                        &effective_dest,
                        "verification failed: destination content mismatch",
                    );
                }
                return Err(MoveError::Io(
                    "verification failed: destination content mismatch".to_string(),
                ));
            }
            Err(e) => {
                tracing::debug!(
                    source = %source,
                    destination = %effective_dest,
                    error = %e,
                    "transactional copy verification errored"
                );
                let _ = fs::remove_file(effective_dest.as_std_path());
                if let Some(j) = journal {
                    j.record_error(JournalOp::Failed, source, &effective_dest, &e.to_string());
                }
                return Err(e);
            }
        }

        if let Some(j) = journal {
            j.record_op(JournalOp::CopyVerified, source, &effective_dest);
        }

        if let Err(e) = fs::remove_file(source.as_std_path()) {
            tracing::debug!(
                source = %source,
                destination = %effective_dest,
                error = %e,
                "transactional copy failed to delete source after verification"
            );
            if let Some(j) = journal {
                j.record_error(
                    JournalOp::Failed,
                    source,
                    &effective_dest,
                    &format!("copied and verified but failed to remove source: {e}"),
                );
            }
            return Err(MoveError::Io(format!(
                "copied and verified but failed to remove source: {e}"
            )));
        }

        if let Some(j) = journal {
            j.record_op(JournalOp::SourceDeleted, source, source);
            j.record_op(JournalOp::Completed, source, &effective_dest);
        }
    }

    tracing::debug!(
        source = %source,
        destination = %effective_dest,
        action = ?action,
        "transactional move complete"
    );
    Ok((true, action))
}

pub fn verify_file_copy(source: &Utf8Path, dest: &Utf8Path) -> Result<bool, MoveError> {
    let src_meta = fs::metadata(source)
        .map_err(|e| MoveError::Io(format!("failed to stat source for verification: {e}")))?;
    let dst_meta = fs::metadata(dest)
        .map_err(|e| MoveError::Io(format!("failed to stat destination for verification: {e}")))?;

    if src_meta.len() != dst_meta.len() {
        return Ok(false);
    }
    if src_meta.len() < VERIFY_CONTENT_THRESHOLD {
        return Ok(true);
    }

    let src_file = File::open(source.as_std_path())
        .map_err(|e| MoveError::Io(format!("failed to open source for verification: {e}")))?;
    let dst_file = File::open(dest.as_std_path())
        .map_err(|e| MoveError::Io(format!("failed to open destination for verification: {e}")))?;

    let mut src_reader = BufReader::with_capacity(VERIFY_CHUNK_SIZE, src_file);
    let mut dst_reader = BufReader::with_capacity(VERIFY_CHUNK_SIZE, dst_file);
    let mut src_buf = vec![0; VERIFY_CHUNK_SIZE];
    let mut dst_buf = vec![0; VERIFY_CHUNK_SIZE];

    loop {
        let src_read = src_reader
            .read(&mut src_buf)
            .map_err(|e| MoveError::Io(format!("failed to read source for verification: {e}")))?;
        let dst_read = dst_reader.read(&mut dst_buf).map_err(|e| {
            MoveError::Io(format!("failed to read destination for verification: {e}"))
        })?;

        if src_read != dst_read {
            return Ok(false);
        }
        if src_read == 0 {
            return Ok(true);
        }
        if src_buf[..src_read] != dst_buf[..dst_read] {
            return Ok(false);
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreflightResult {
    pub free_space_ok: bool,
    pub path_len_ok: bool,
    pub perms_ok: bool,
    pub errors: Vec<String>,
}

pub fn check_preflight(sources: &[&Utf8Path], destination_dir: &Utf8Path) -> PreflightResult {
    let mut errors = Vec::new();

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

    let probe = destination_dir.join(".rosey_write_probe");
    match fs::write(&probe, b"") {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
        }
        Err(e) => {
            errors.push(format!("Destination is not writable: {e}"));
        }
    }

    let mut total_size: u64 = 0;
    for src in sources {
        if let Ok(meta) = fs::metadata(src) {
            total_size += meta.len();
        }
    }

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
            let buffer = 100 * 1024 * 1024;
            if free_bytes < total_size + buffer {
                errors.push(format!(
                    "Insufficient space: need {}, have {}",
                    crate::format_bytes(total_size + buffer),
                    crate::format_bytes(free_bytes)
                ));
            }
        }
    }

    let dest_prefix_len = destination_dir.as_str().len() + 1;
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

pub fn move_with_sidecars(
    item: &MediaItem,
    destination: &Utf8Path,
    conflict_policy: ConflictPolicy,
    dry_run: bool,
) -> MoveResult {
    move_with_sidecars_journaled(item, destination, conflict_policy, dry_run, None)
}

pub fn move_with_sidecars_journaled(
    item: &MediaItem,
    destination: &Utf8Path,
    conflict_policy: ConflictPolicy,
    dry_run: bool,
    journal: Option<&OperationJournal>,
) -> MoveResult {
    let source = &item.source_path;
    let sidecars = collect_sidecars(item);
    tracing::debug!(
        source = %source,
        destination = %destination,
        sidecars = sidecars.len(),
        conflict_policy = ?conflict_policy,
        dry_run,
        journaled = journal.is_some(),
        "move with sidecars started"
    );

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

    let dest_dir = destination.parent().unwrap_or_else(|| Utf8Path::new(""));
    let preflight =
        check_preflight(&all_sources.iter().map(|p| p.as_ref()).collect::<Vec<_>>(), dest_dir);

    if !preflight.free_space_ok || !preflight.path_len_ok || !preflight.perms_ok {
        tracing::debug!(
            source = %source,
            destination = %destination,
            errors = ?preflight.errors,
            "move with sidecars preflight failed"
        );
        if let Some(j) = journal {
            j.record_error(JournalOp::Failed, source, destination, "preflight checks failed");
        }
        result.errors = preflight.errors;
        return result;
    }

    match move_file_transactional_journaled(source, destination, conflict_policy, dry_run, journal)
    {
        Ok((true, action)) => {
            tracing::debug!(
                source = %source,
                destination = %destination,
                action = ?action,
                "primary media move complete"
            );
            record_action(&mut result, action, destination);
            moved_files.push(destination.to_path_buf());
        }
        Ok((false, _)) | Err(_) => {
            tracing::debug!(source = %source, destination = %destination, "primary media move failed");
            result.errors.push(format!("Failed to move {source}"));
            if let Some(j) = journal {
                j.record_error(JournalOp::Failed, source, destination, "move failed");
            }
            return result;
        }
    }

    let dest_parent = destination.parent().unwrap_or_else(|| Utf8Path::new(""));
    let dest_stem = destination.file_stem().unwrap_or("");

    for sidecar in &sidecars {
        let sidecar_dest = sidecar_destination(source, sidecar, dest_parent, dest_stem);
        tracing::debug!(
            sidecar = %sidecar,
            destination = %sidecar_dest,
            "sidecar move started"
        );

        match move_file_transactional_journaled(
            sidecar,
            &sidecar_dest,
            conflict_policy,
            dry_run,
            journal,
        ) {
            Ok((true, action)) => {
                tracing::debug!(
                    sidecar = %sidecar,
                    destination = %sidecar_dest,
                    action = ?action,
                    "sidecar move complete"
                );
                record_action(&mut result, action, &sidecar_dest);
                moved_files.push(sidecar_dest);
            }
            Ok((false, _)) | Err(_) => {
                tracing::debug!(
                    sidecar = %sidecar,
                    destination = %sidecar_dest,
                    moved_files = moved_files.len(),
                    dry_run,
                    "sidecar move failed"
                );
                if !dry_run {
                    for moved in &moved_files {
                        let _ = fs::remove_file(moved.as_std_path());
                    }
                    result.rollback_performed = true;
                    if let Some(j) = journal {
                        j.record_error(
                            JournalOp::RolledBack,
                            sidecar,
                            &sidecar_dest,
                            "rollback after sidecar move failure",
                        );
                    }
                }
                result.errors.push(format!("Failed to move sidecar {sidecar}, rolled back"));
                return result;
            }
        }
    }

    result.success = true;
    tracing::debug!(
        source = %source,
        destination = %destination,
        moved = result.moved.len(),
        skipped = result.skipped.len(),
        replaced = result.replaced.len(),
        kept_both = result.kept_both.len(),
        "move with sidecars complete"
    );
    result
}

fn collect_sidecars(item: &MediaItem) -> Vec<Utf8PathBuf> {
    let source = &item.source_path;
    let mut seen = BTreeSet::new();
    let mut sidecars = Vec::new();
    let discovered = crate::discover_sidecars(source);

    for sidecar in item.sidecars.iter().chain(discovered.iter()) {
        if sidecar == source
            || !sidecar.is_file()
            || !is_sidecar_path(sidecar)
            || !seen.insert(sidecar.clone())
        {
            continue;
        }
        sidecars.push(sidecar.clone());
    }

    tracing::debug!(source = %source, sidecars = sidecars.len(), "sidecars collected");
    sidecars
}

fn sidecar_destination(
    source: &Utf8Path,
    sidecar: &Utf8Path,
    dest_parent: &Utf8Path,
    dest_stem: &str,
) -> Utf8PathBuf {
    let source_parent = source.parent().unwrap_or_else(|| Utf8Path::new(""));
    let same_directory = sidecar.parent() == Some(source_parent);
    let same_stem = sidecar.file_stem() == source.file_stem();

    if same_directory && same_stem {
        let sidecar_ext = sidecar.extension().unwrap_or("");
        return if sidecar_ext.is_empty() {
            dest_parent.join(dest_stem)
        } else {
            dest_parent.join(format!("{dest_stem}.{sidecar_ext}"))
        };
    }

    if let Ok(relative) = sidecar.strip_prefix(source_parent) {
        return dest_parent.join(relative);
    }

    sidecar
        .file_name()
        .map(|name| dest_parent.join(name))
        .unwrap_or_else(|| dest_parent.join("sidecar"))
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowMoveResult {
    pub show_title: String,
    pub show_year: Option<u16>,
    pub episode_results: Vec<EpisodeMoveResult>,
    pub asset_results: Vec<AssetMoveResult>,
    pub success: bool,
    pub total_moved: usize,
    pub total_skipped: usize,
    pub total_replaced: usize,
    pub total_kept_both: usize,
    pub total_errors: usize,
    pub rollback_performed: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMoveResult {
    pub season: u16,
    pub episode_numbers: Vec<u16>,
    pub source_path: Utf8PathBuf,
    pub destination: Utf8PathBuf,
    pub move_result: MoveResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMoveResult {
    pub source_path: Utf8PathBuf,
    pub destination: Utf8PathBuf,
    pub asset_kind: ShowAssetKind,
    pub success: bool,
    pub error: Option<String>,
}

pub fn move_show(
    show: &TvShow,
    planner: &rosey_core::Planner,
    conflict_policy: ConflictPolicy,
    dry_run: bool,
) -> ShowMoveResult {
    move_show_journaled(show, planner, conflict_policy, dry_run, None)
}

pub fn move_show_journaled(
    show: &TvShow,
    planner: &rosey_core::Planner,
    conflict_policy: ConflictPolicy,
    dry_run: bool,
    journal: Option<&OperationJournal>,
) -> ShowMoveResult {
    let mut result = ShowMoveResult {
        show_title: show.title.clone(),
        show_year: show.year,
        episode_results: Vec::new(),
        asset_results: Vec::new(),
        success: false,
        total_moved: 0,
        total_skipped: 0,
        total_replaced: 0,
        total_kept_both: 0,
        total_errors: 0,
        rollback_performed: false,
        errors: Vec::new(),
    };

    let destinations = planner.plan_show(show);
    if destinations.is_empty() {
        result.errors.push("No destinations to move".to_string());
        return result;
    }

    if !dry_run {
        for dest in &destinations {
            if let Some(parent) = dest.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent.as_std_path()) {
                        let error = format!("Failed to create directory {}: {e}", parent);
                        tracing::debug!(directory = %parent, error = %e, "move_show directory creation failed");
                        if let Some(j) = journal {
                            j.record_error(JournalOp::Failed, &Utf8PathBuf::new(), dest, &error);
                        }
                        result.errors.push(error);
                        result.total_errors += 1;
                    }
                }
            }
        }
    }

    for season in &show.seasons {
        for episode in &season.episodes {
            let dest = planner.plan_destination(&episode.item);
            let move_result = move_with_sidecars_journaled(
                &episode.item,
                &dest,
                conflict_policy,
                dry_run,
                journal,
            );

            result.total_moved += move_result.moved.len();
            result.total_skipped += move_result.skipped.len();
            result.total_replaced += move_result.replaced.len();
            result.total_kept_both += move_result.kept_both.len();
            result.total_errors += move_result.errors.len();
            if move_result.rollback_performed {
                result.rollback_performed = true;
            }
            result.errors.extend(move_result.errors.clone());

            result.episode_results.push(EpisodeMoveResult {
                season: episode.item.season.unwrap_or(0),
                episode_numbers: episode.item.episodes.clone(),
                source_path: episode.item.source_path.clone(),
                destination: dest,
                move_result,
            });
        }
    }

    for asset in &show.show_assets {
        if !asset.source_path.exists() {
            let error = format!("Asset source does not exist: {}", asset.source_path);
            tracing::debug!(
                source = %asset.source_path,
                destination = %asset.destination,
                kind = ?asset.asset_kind,
                "asset source missing"
            );
            result.errors.push(error);
            result.total_errors += 1;
            result.asset_results.push(AssetMoveResult {
                source_path: asset.source_path.clone(),
                destination: asset.destination.clone(),
                asset_kind: asset.asset_kind,
                success: false,
                error: Some(format!("Source missing: {}", asset.source_path)),
            });
            continue;
        }

        if let Some(parent) = asset.destination.parent() {
            if !parent.exists() && !dry_run {
                if let Err(e) = fs::create_dir_all(parent.as_std_path()) {
                    let error = format!("Failed to create asset directory {}: {e}", parent);
                    tracing::debug!(directory = %parent, error = %e, "asset directory creation failed");
                    result.errors.push(error.clone());
                    result.total_errors += 1;
                    result.asset_results.push(AssetMoveResult {
                        source_path: asset.source_path.clone(),
                        destination: asset.destination.clone(),
                        asset_kind: asset.asset_kind,
                        success: false,
                        error: Some(error),
                    });
                    continue;
                }
            }
        }

        let (success, action) = match move_file_transactional_journaled(
            &asset.source_path,
            &asset.destination,
            conflict_policy,
            dry_run,
            journal,
        ) {
            Ok((true, action)) => (true, action),
            Ok((false, _)) | Err(_) => (false, MoveAction::Moved),
        };

        if success {
            result.total_moved += 1;
            match action {
                MoveAction::Moved | MoveAction::WouldMove => {}
                MoveAction::Skipped => result.total_skipped += 1,
                MoveAction::Replaced => result.total_replaced += 1,
                MoveAction::KeptBoth => result.total_kept_both += 1,
            }
        } else {
            result.total_errors += 1;
        }

        result.asset_results.push(AssetMoveResult {
            source_path: asset.source_path.clone(),
            destination: asset.destination.clone(),
            asset_kind: asset.asset_kind,
            success,
            error: if success {
                None
            } else {
                Some(format!("Failed to move asset: {}", asset.source_path))
            },
        });
    }

    for season in &show.seasons {
        for asset in &season.season_assets {
            if !asset.source_path.exists() {
                let error = format!("Season asset source does not exist: {}", asset.source_path);
                result.errors.push(error);
                result.total_errors += 1;
                result.asset_results.push(AssetMoveResult {
                    source_path: asset.source_path.clone(),
                    destination: asset.destination.clone(),
                    asset_kind: asset.asset_kind,
                    success: false,
                    error: Some(format!("Source missing: {}", asset.source_path)),
                });
                continue;
            }

            if let Some(parent) = asset.destination.parent() {
                if !parent.exists() && !dry_run {
                    if let Err(e) = fs::create_dir_all(parent.as_std_path()) {
                        let error =
                            format!("Failed to create season asset directory {}: {e}", parent);
                        result.errors.push(error.clone());
                        result.total_errors += 1;
                        result.asset_results.push(AssetMoveResult {
                            source_path: asset.source_path.clone(),
                            destination: asset.destination.clone(),
                            asset_kind: asset.asset_kind,
                            success: false,
                            error: Some(error),
                        });
                        continue;
                    }
                }
            }

            let (success, action) = match move_file_transactional_journaled(
                &asset.source_path,
                &asset.destination,
                conflict_policy,
                dry_run,
                journal,
            ) {
                Ok((true, action)) => (true, action),
                Ok((false, _)) | Err(_) => (false, MoveAction::Moved),
            };

            if success {
                result.total_moved += 1;
                match action {
                    MoveAction::Moved | MoveAction::WouldMove => {}
                    MoveAction::Skipped => result.total_skipped += 1,
                    MoveAction::Replaced => result.total_replaced += 1,
                    MoveAction::KeptBoth => result.total_kept_both += 1,
                }
            } else {
                result.total_errors += 1;
            }

            result.asset_results.push(AssetMoveResult {
                source_path: asset.source_path.clone(),
                destination: asset.destination.clone(),
                asset_kind: asset.asset_kind,
                success,
                error: if success {
                    None
                } else {
                    Some(format!("Failed to move season asset: {}", asset.source_path))
                },
            });
        }
    }

    result.success = result.total_errors == 0;

    tracing::debug!(
        show_title = %show.title,
        total_moved = result.total_moved,
        total_skipped = result.total_skipped,
        total_errors = result.total_errors,
        "move_show complete"
    );

    result
}
