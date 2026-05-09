use thiserror::Error;

pub type ApiTokenResult<T> = Result<T, ApiTokenError>;

#[derive(Debug, Error)]
pub enum ApiTokenError {
    #[error("api token not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("api token conflict: {0}")]
    Conflict(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
