use constants::pagination::PAGE_INDEX_OFFSET;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{User, UserId},
};

use crate::{Database, StorageError, StorageResult, rbac::RoleRecord};

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
        ensure_role_exists(&mut db, &user.role).await?;
        toasty::create!(UserRecord {
            id: self.database.next_id(),
            username: user.username,
            password_hash: user.password_hash,
            email: user.email,
            role: user.role,
            is_active: user.is_active,
            is_deleted: false,
            last_login_at: None,
            auth_source: UserRecord::local_auth_source(),
            email_verified: false,
        })
        .exec(&mut db)
        .await
        .map(User::from)
        .map_err(StorageError::from)
    }

    pub async fn replace(&self, id: UserId, user: UserRecordInput) -> StorageResult<User> {
        let mut db = self.database.connection();
        ensure_role_exists(&mut db, &user.role).await?;
        let mut record = self.find_record_by_id(&id).await?.ok_or(StorageError::NotFound)?;
        record
            .update()
            .username(user.username)
            .password_hash(user.password_hash)
            .email(user.email)
            .role(user.role)
            .is_active(user.is_active)
            .exec(&mut db)
            .await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: UserId) -> StorageResult<()> {
        let mut db = self.database.connection();
        let mut record = self.find_record_by_id(&id).await?.ok_or(StorageError::NotFound)?;
        record.update().is_deleted(true).exec(&mut db).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: UserId) -> StorageResult<Option<User>> {
        self.find_record_by_id(&id).await.map(|record| record.map(User::from))
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        let mut db = self.database.connection();
        UserRecord::filter(active_user_filter().and(UserRecord::fields().email().eq(email)))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.map(User::from))
            .map_err(StorageError::from)
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<UserAuthRecord>> {
        let mut db = self.database.connection();
        UserRecord::filter(active_user_filter().and(UserRecord::fields().username().eq(username)))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.map(UserRecord::into_auth))
            .map_err(StorageError::from)
    }

    pub async fn find_auth_by_email(&self, email: &str) -> StorageResult<Option<UserAuthRecord>> {
        let mut db = self.database.connection();
        UserRecord::filter(active_user_filter().and(UserRecord::fields().email().eq(email)))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.map(UserRecord::into_auth))
            .map_err(StorageError::from)
    }

    pub async fn record_login(&self, id: UserId) -> StorageResult<()> {
        let mut db = self.database.connection();
        let mut record = self.find_record_by_id(&id).await?.ok_or(StorageError::NotFound)?;
        record.update().last_login_at(jiff::Timestamp::now()).exec(&mut db).await?;
        Ok(())
    }

    pub async fn list(&self, page: PageRequest) -> StorageResult<Page<User>> {
        let request = PageSliceRequest {
            offset: (page.page - PAGE_INDEX_OFFSET) * page.page_size,
            limit: page.page_size,
            page: page.page,
            page_size: page.page_size,
        };
        self.list_slice(request).await
    }

    pub async fn list_slice(&self, request: PageSliceRequest) -> StorageResult<Page<User>> {
        let mut db = self.database.connection();
        let total = UserRecord::filter(active_user_filter()).count().exec(&mut db).await?;
        let items = UserRecord::filter(active_user_filter())
            .order_by(UserRecord::fields().created_at().asc())
            .limit(request.limit as usize)
            .offset(request.offset as usize)
            .exec(&mut db)
            .await?;
        Ok(Page {
            items: items.into_iter().map(User::from).collect(),
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn find_record_by_id(&self, id: &UserId) -> StorageResult<Option<UserRecord>> {
        let mut db = self.database.connection();
        UserRecord::filter(active_user_filter().and(UserRecord::fields().id().eq(id.0.as_str())))
            .first()
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }
}

fn active_user_filter() -> toasty::stmt::Expr<bool> {
    UserRecord::fields().is_deleted().eq(false)
}

async fn ensure_role_exists(db: &mut toasty::Db, role: &str) -> StorageResult<()> {
    let exists = RoleRecord::filter(RoleRecord::fields().code().eq(role)).first().exec(db).await?.is_some();
    if exists {
        return Ok(());
    }

    Err(StorageError::Conflict(format!("role does not exist: {role}")))
}
