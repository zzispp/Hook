use thiserror::Error;

pub type ProviderResult<T> = Result<T, ProviderError>;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider resource not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("provider conflict: {0}")]
    Conflict(String),
    #[error("secret error: {0}")]
    Secret(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
