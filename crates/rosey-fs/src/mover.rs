use camino::{Utf8Path, Utf8PathBuf};
use rosey_core::ConflictPolicy;
use serde::{Deserialize, Serialize};
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

pub fn plan_single_file_move(request: &MoveRequest) -> Result<MoveOutcome, MoveError> {
    if request.dry_run {
        return Ok(MoveOutcome {
            success: true,
            action: MoveAction::WouldMove,
            source: request.source.clone(),
            destination: request.destination.clone(),
            error: None,
        });
    }

    if !Utf8Path::new(request.source.as_str()).exists() {
        return Err(MoveError::SourceMissing(request.source.clone()));
    }

    if Utf8Path::new(request.destination.as_str()).exists()
        && request.conflict_policy == ConflictPolicy::Skip
    {
        return Ok(MoveOutcome {
            success: true,
            action: MoveAction::Skipped,
            source: request.source.clone(),
            destination: request.destination.clone(),
            error: None,
        });
    }

    // Real move/copy execution intentionally comes later.
    // See ADR-0005 and design/SPEC.md.
    Ok(MoveOutcome {
        success: true,
        action: MoveAction::WouldMove,
        source: request.source.clone(),
        destination: request.destination.clone(),
        error: Some("execute mode not implemented in starter skeleton".to_string()),
    })
}
