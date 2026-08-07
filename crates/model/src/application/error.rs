use thiserror::Error;

pub type ModelResult<T> = Result<T, ModelError>;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("resource not found")]
    NotFound,
    #[error("resource conflict: {0}")]
    Conflict(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
