use constants::pagination::PAGE_INDEX_OFFSET;
use types::user::{Page, PageRequest, User, UserId};

use crate::{Database, StorageError, StorageResult};

use super::{UserAuthRecord, UserRecord, UserRecordInput};

#[derive(Clone)]
pub struct UserStore {
    database: Database,
}

impl UserStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, user: UserRecordInput) -> StorageResult<User> {
        let mut db = self.database.connection();
        toasty::create!(UserRecord {
            username: user.username,
            password_hash: user.password_hash,
            email: user.email,
            role: user.role,
            status: user.status,
        })
        .exec(&mut db)
        .await
        .map(User::from)
        .map_err(StorageError::from)
    }

    pub async fn replace(&self, id: UserId, user: UserRecordInput) -> StorageResult<User> {
        let mut db = self.database.connection();
        let mut record = self.find_record_by_id(id).await?.ok_or(StorageError::NotFound)?;
        record
            .update()
            .username(user.username)
            .password_hash(user.password_hash)
            .email(user.email)
            .role(user.role)
            .status(user.status)
            .exec(&mut db)
            .await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: UserId) -> StorageResult<()> {
        let mut db = self.database.connection();
        let record = self.find_record_by_id(id).await?.ok_or(StorageError::NotFound)?;
        record.delete().exec(&mut db).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: UserId) -> StorageResult<Option<User>> {
        self.find_record_by_id(id).await.map(|record| record.map(User::from))
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        let mut db = self.database.connection();
        UserRecord::filter(UserRecord::fields().email().eq(email))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.map(User::from))
            .map_err(StorageError::from)
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<UserAuthRecord>> {
        let mut db = self.database.connection();
        UserRecord::filter(UserRecord::fields().username().eq(username))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.map(UserRecord::into_auth))
            .map_err(StorageError::from)
    }

    pub async fn list(&self, page: PageRequest) -> StorageResult<Page<User>> {
        let mut db = self.database.connection();
        let total = UserRecord::all().count().exec(&mut db).await?;
        let items = UserRecord::all()
            .order_by(UserRecord::fields().id().asc())
            .limit(page.page_size as usize)
            .offset(((page.page - PAGE_INDEX_OFFSET) * page.page_size) as usize)
            .exec(&mut db)
            .await?;
        Ok(Page {
            items: items.into_iter().map(User::from).collect(),
            total,
            page: page.page,
            page_size: page.page_size,
        })
    }

    async fn find_record_by_id(&self, id: UserId) -> StorageResult<Option<UserRecord>> {
        let mut db = self.database.connection();
        UserRecord::filter(UserRecord::fields().id().eq(id.0))
            .first()
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }
}
