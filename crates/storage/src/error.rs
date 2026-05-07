use thiserror::Error;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("resource not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(String),
}

impl From<toasty::Error> for StorageError {
    fn from(value: toasty::Error) -> Self {
        Self::Database(value.to_string())
    }
}
