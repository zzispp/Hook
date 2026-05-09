use thiserror::Error;

pub type SettingResult<T> = Result<T, SettingError>;

#[derive(Debug, Error)]
pub enum SettingError {
    #[error("settings not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}
