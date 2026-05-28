use thiserror::Error;

pub type PaymentResult<T> = Result<T, PaymentError>;

#[derive(Debug, Error)]
pub enum PaymentError {
    #[error("payment channel is not configured: {0}")]
    MissingConfig(String),
    #[error("invalid payment config: {0}")]
    InvalidConfig(String),
    #[error("invalid payment request: {0}")]
    InvalidRequest(String),
    #[error("unsupported payment capability: {0}")]
    Unsupported(String),
    #[error("payment verification failed: {0}")]
    VerificationFailed(String),
    #[error("payment provider error: {0}")]
    Provider(String),
}
