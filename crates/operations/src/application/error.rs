use thiserror::Error;

pub type OperationsResult<T> = Result<T, OperationsError>;

#[derive(Debug, Error)]
pub enum OperationsError {
    #[error("forbidden")]
    Forbidden,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found")]
    NotFound,
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
