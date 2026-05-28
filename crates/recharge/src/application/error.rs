use thiserror::Error;

pub type RechargeResult<T> = Result<T, RechargeError>;

#[derive(Debug, Error)]
pub enum RechargeError {
    #[error("recharge resource not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("forbidden")]
    Forbidden,
    #[error("recharge conflict: {0}")]
    Conflict(String),
    #[error("payment error: {0}")]
    Payment(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
