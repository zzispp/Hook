use constants::pagination::PAGE_INDEX_OFFSET;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select, Set};
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{User, UserId},
};

use crate::{
    Database, StorageError, StorageResult,
    rbac::role_records,
    user::record::ActiveModel as UserActiveModel,
    user::{UserColumn, UserEntity as Users},
};

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
        ensure_role_exists(self.database.connection(), &user.role).await?;
        UserActiveModel {
            id: Set(self.database.next_id()),
            username: Set(user.username),
            password_hash: Set(user.password_hash),
            email: Set(user.email),
            role: Set(user.role),
            is_active: Set(user.is_active),
            is_deleted: Set(false),
            last_login_at: Set(None),
            auth_source: Set(UserRecord::local_auth_source()),
            email_verified: Set(false),
            ..Default::default()
        }
        .insert(self.database.connection())
        .await
        .map(User::from)
        .map_err(StorageError::from)
    }

    pub async fn replace(&self, id: UserId, user: UserRecordInput) -> StorageResult<User> {
        ensure_role_exists(self.database.connection(), &user.role).await?;
        let record = self.find_record_by_id(&id).await?.ok_or(StorageError::NotFound)?;
        let mut active: UserActiveModel = record.into();
        active.username = Set(user.username);
        active.password_hash = Set(user.password_hash);
        active.email = Set(user.email);
        active.role = Set(user.role);
        active.is_active = Set(user.is_active);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: UserId) -> StorageResult<()> {
        let record = self.find_record_by_id(&id).await?.ok_or(StorageError::NotFound)?;
        let mut active: UserActiveModel = record.into();
        active.is_deleted = Set(true);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: UserId) -> StorageResult<Option<User>> {
        self.find_record_by_id(&id).await.map(|record| record.map(User::from))
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        self.find_record(UserColumn::Email.eq(email).into()).await.map(|record| record.map(User::from))
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<UserAuthRecord>> {
        self.find_record(UserColumn::Username.eq(username).into())
            .await
            .map(|record| record.map(UserRecord::into_auth))
    }

    pub async fn find_auth_by_email(&self, email: &str) -> StorageResult<Option<UserAuthRecord>> {
        self.find_record(UserColumn::Email.eq(email).into())
            .await
            .map(|record| record.map(UserRecord::into_auth))
    }

    pub async fn record_login(&self, id: UserId) -> StorageResult<()> {
        let record = self.find_record_by_id(&id).await?.ok_or(StorageError::NotFound)?;
        let mut active: UserActiveModel = record.into();
        active.last_login_at = Set(Some(time::OffsetDateTime::now_utc()));
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        Ok(())
    }

    pub async fn list(&self, page: PageRequest) -> StorageResult<Page<User>> {
        self.list_slice(PageSliceRequest {
            offset: (page.page - PAGE_INDEX_OFFSET) * page.page_size,
            limit: page.page_size,
            page: page.page,
            page_size: page.page_size,
        })
        .await
    }

    pub async fn list_slice(&self, request: PageSliceRequest) -> StorageResult<Page<User>> {
        let total = active_users().count(self.database.connection()).await?;
        let items = active_users()
            .order_by_asc(UserColumn::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        Ok(Page {
            items: items.into_iter().map(User::from).collect(),
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn find_record_by_id(&self, id: &UserId) -> StorageResult<Option<UserRecord>> {
        self.find_record(UserColumn::Id.eq(id.0.as_str()).into()).await
    }

    async fn find_record(&self, filter: Condition) -> StorageResult<Option<UserRecord>> {
        active_users().filter(filter).one(self.database.connection()).await.map_err(StorageError::from)
    }
}

fn active_users() -> Select<Users> {
    Users::find().filter(UserColumn::IsDeleted.eq(false))
}

async fn ensure_role_exists(db: &DatabaseConnection, role: &str) -> StorageResult<()> {
    let exists = role_records::Entity::find_by_id(role.to_owned()).one(db).await?.is_some();
    if exists {
        return Ok(());
    }
    Err(StorageError::Conflict(format!("role does not exist: {role}")))
}
