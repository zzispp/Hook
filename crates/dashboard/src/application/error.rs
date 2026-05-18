use thiserror::Error;

pub type DashboardResult<T> = Result<T, DashboardError>;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
