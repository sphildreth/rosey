use thiserror::Error;

#[derive(Debug, Error)]
pub enum RoseyError {
    #[error("invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("parse failed: {0}")]
    ParseFailed(String),
}
