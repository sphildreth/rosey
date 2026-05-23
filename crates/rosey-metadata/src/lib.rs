use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("provider is not configured: {0}")]
    ProviderNotConfigured(String),

    #[error("provider request failed: {0}")]
    RequestFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderMatch {
    pub provider: String,
    pub id: String,
    pub canonical_title: String,
    pub year: Option<u16>,
}

pub trait MetadataProvider {
    fn provider_name(&self) -> &'static str;

    fn movie_by_id(&self, _id: &str) -> Result<Option<ProviderMatch>, MetadataError> {
        Ok(None)
    }

    fn tv_by_id(&self, _id: &str) -> Result<Option<ProviderMatch>, MetadataError> {
        Ok(None)
    }
}
