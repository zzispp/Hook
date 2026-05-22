use thiserror::Error;

pub type SchedulerResult<T> = Result<T, SchedulerError>;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("scheduled task not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}

impl From<storage::StorageError> for SchedulerError {
    fn from(value: storage::StorageError) -> Self {
        match value {
            storage::StorageError::NotFound => Self::NotFound("scheduled task".into()),
            storage::StorageError::Conflict(message) | storage::StorageError::Database(message) => Self::Infrastructure(message),
        }
    }
}
