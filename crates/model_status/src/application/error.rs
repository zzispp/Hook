use thiserror::Error;

pub type ModelStatusResult<T> = Result<T, ModelStatusError>;

#[derive(Debug, Error)]
pub enum ModelStatusError {
    #[error("model status check not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("model status conflict: {0}")]
    Conflict(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
