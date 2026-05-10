use thiserror::Error;

pub type I18nResult<T> = Result<T, I18nError>;

#[derive(Debug, Error)]
pub enum I18nError {
    #[error("translation resource not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("translation conflict: {0}")]
    Conflict(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
