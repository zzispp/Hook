use async_trait::async_trait;
use storage::{
    Database, StorageError,
    user::{UserRecordInput as StorageUserRecordInput, UserStore},
};

use crate::application::{AppError, AppResult, ReplaceUserRecord, UserAuthRecord, UserRepository};
use types::user::{Page, PageRequest, User, UserId};

#[derive(Clone)]
pub struct StorageUserRepository {
    store: UserStore,
}

impl StorageUserRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: UserStore::new(database),
        }
    }
}

#[async_trait]
impl UserRepository for StorageUserRepository {
    async fn create(&self, user: ReplaceUserRecord) -> AppResult<User> {
        self.store.create(storage_record_input(user)).await.map_err(storage_error)
    }

    async fn replace(&self, id: UserId, user: ReplaceUserRecord) -> AppResult<User> {
        self.store.replace(id, storage_record_input(user)).await.map_err(storage_error)
    }

    async fn delete(&self, id: UserId) -> AppResult<()> {
        self.store.delete(id).await.map_err(storage_error)
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        self.store.find_by_id(id).await.map_err(storage_error)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        self.store.find_by_email(email).await.map_err(storage_error)
    }

    async fn find_auth_by_username(&self, username: &str) -> AppResult<Option<UserAuthRecord>> {
        self.store
            .find_auth_by_username(username)
            .await
            .map(|record| record.map(user_auth_record))
            .map_err(storage_error)
    }

    async fn list(&self, page: PageRequest) -> AppResult<Page<User>> {
        self.store.list(page).await.map_err(storage_error)
    }
}

fn storage_record_input(record: ReplaceUserRecord) -> StorageUserRecordInput {
    StorageUserRecordInput {
        username: record.username,
        password_hash: record.password_hash,
        email: record.email,
        role: record.role,
        status: record.status,
    }
}

fn user_auth_record(record: storage::user::UserAuthRecord) -> UserAuthRecord {
    UserAuthRecord {
        user: record.user,
        password_hash: record.password_hash,
    }
}

fn storage_error(error: StorageError) -> AppError {
    match error {
        StorageError::NotFound => AppError::NotFound,
        StorageError::Database(message) => AppError::Infrastructure(message),
    }
}
