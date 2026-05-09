use thiserror::Error;

pub type WalletResult<T> = Result<T, WalletError>;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("wallet not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("forbidden")]
    Forbidden,
    #[error("wallet conflict: {0}")]
    Conflict(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
