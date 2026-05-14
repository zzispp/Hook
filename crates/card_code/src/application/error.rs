use thiserror::Error;

pub type CardCodeResult<T> = Result<T, CardCodeError>;

#[derive(Debug, Error)]
pub enum CardCodeError {
    #[error("card code not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("card code conflict: {0}")]
    Conflict(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
