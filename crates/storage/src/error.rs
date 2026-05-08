use thiserror::Error;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("resource not found")]
    NotFound,
    #[error("resource conflict: {0}")]
    Conflict(String),
    #[error("database error: {0}")]
    Database(String),
}

impl From<sea_orm::DbErr> for StorageError {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(value: serde_json::Error) -> Self {
        Self::Database(value.to_string())
    }
}
