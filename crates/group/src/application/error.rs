use thiserror::Error;

pub type GroupResult<T> = Result<T, GroupError>;

#[derive(Debug, Error)]
pub enum GroupError {
    #[error("billing group not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("billing group conflict: {0}")]
    Conflict(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
