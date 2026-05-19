use constants::pagination::PAGE_INDEX_OFFSET;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{User, UserId, UserListFilters},
};

use crate::{
    Database, StorageError, StorageResult, json,
    rbac::role_records,
    user::UserColumn,
    user::password_reset_tokens::{self, ActiveModel as PasswordResetTokenActiveModel},
    user::record::ActiveModel as UserActiveModel,
};

use super::{
    PasswordResetTokenRecord, PasswordResetTokenRecordInput, UserAuthRecord, UserRecord, UserRecordInput,
    query::{active_users, filtered_users},
    tokens::{password_reset_token_active_model, password_reset_token_record},
    user_mutations::{delete_user_api_tokens, required_password_hash, set_wallet_limit_mode},
};

#[derive(Clone)]
pub struct UserStore {
    pub(super) database: Database,
}

impl UserStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, user: UserRecordInput) -> StorageResult<User> {
        ensure_role_exists(self.database.connection(), &user.role).await?;
        let now = time::OffsetDateTime::now_utc();
        UserActiveModel {
            id: Set(self.database.next_id()),
            username: Set(user.username),
            password_hash: Set(required_password_hash(user.password_hash)?),
            email: Set(user.email),
            role: Set(user.role),
            is_active: Set(user.is_active),
            is_deleted: Set(false),
            allowed_model_ids: Set(json::encode_required(&user.allowed_model_ids)?),
            allowed_provider_ids: Set(json::encode_required(&user.allowed_provider_ids)?),
            last_login_at: Set(None),
            auth_source: Set(UserRecord::local_auth_source()),
            email_verified: Set(user.email_verified.unwrap_or(false)),
            rate_limit_rpm: Set(user.rate_limit_rpm),
            quota_mode: Set(user.quota_mode),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await
        .map_err(StorageError::from)?
        .into_domain()
    }

    pub async fn replace(&self, id: UserId, user: UserRecordInput) -> StorageResult<User> {
        ensure_role_exists(self.database.connection(), &user.role).await?;
        let tx = self.database.connection().begin().await?;
        let record = self.find_record_by_id_in_tx(&id, &tx).await?.ok_or(StorageError::NotFound)?;
        let mut active: UserActiveModel = record.into();
        let quota_mode = user.quota_mode.clone();
        active.username = Set(user.username);
        if let Some(password_hash) = user.password_hash {
            active.password_hash = Set(password_hash);
        }
        active.email = Set(user.email);
        active.role = Set(user.role);
        active.is_active = Set(user.is_active);
        active.allowed_model_ids = Set(json::encode_required(&user.allowed_model_ids)?);
        active.allowed_provider_ids = Set(json::encode_required(&user.allowed_provider_ids)?);
        active.rate_limit_rpm = Set(user.rate_limit_rpm);
        active.quota_mode = Set(user.quota_mode);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(&tx).await?;
        set_wallet_limit_mode(&tx, &id.0, &quota_mode).await?;
        tx.commit().await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn create_password_reset_token(&self, input: PasswordResetTokenRecordInput) -> StorageResult<PasswordResetTokenRecord> {
        PasswordResetTokenActiveModel {
            id: Set(self.database.next_id()),
            user_id: Set(input.user_id),
            token_hash: Set(input.token_hash),
            expires_at: Set(input.expires_at),
            consumed_at: Set(None),
            created_at: Set(time::OffsetDateTime::now_utc()),
        }
        .insert(self.database.connection())
        .await
        .map(password_reset_token_record)
        .map_err(StorageError::from)
    }

    pub async fn consume_password_reset_token(&self, token_hash: &str, password_hash: &str, now: time::OffsetDateTime) -> StorageResult<Option<User>> {
        let tx = self.database.connection().begin().await?;
        let Some(token) = self.find_reset_token_in_tx(token_hash, &tx).await? else {
            tx.commit().await?;
            return Ok(None);
        };
        if token.consumed_at.is_some() || token.expires_at <= now {
            tx.commit().await?;
            return Ok(None);
        }
        let user_id = UserId(token.user_id.clone());
        let record = self.find_record_by_id_in_tx(&user_id, &tx).await?.ok_or(StorageError::NotFound)?;
        let mut user_active: UserActiveModel = record.into();
        user_active.password_hash = Set(password_hash.to_owned());
        user_active.updated_at = Set(now);
        user_active.update(&tx).await?;

        let mut token_active = password_reset_token_active_model(token);
        token_active.consumed_at = Set(Some(now));
        token_active.update(&tx).await?;
        tx.commit().await?;
        self.find_by_id(user_id).await
    }

    pub async fn delete(&self, id: UserId) -> StorageResult<()> {
        let tx = self.database.connection().begin().await?;
        let record = self.find_record_by_id_in_tx(&id, &tx).await?.ok_or(StorageError::NotFound)?;
        delete_user_api_tokens(&tx, &id.0).await?;
        let mut active: UserActiveModel = record.into();
        active.is_deleted = Set(true);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(&tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: UserId) -> StorageResult<Option<User>> {
        self.find_record_by_id(&id).await?.map(UserRecord::into_domain).transpose()
    }

    pub async fn find_by_ids(&self, ids: &[String]) -> StorageResult<Vec<User>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        active_users()
            .filter(UserColumn::Id.is_in(ids.iter().cloned()))
            .all(self.database.connection())
            .await
            .map_err(StorageError::from)?
            .into_iter()
            .map(UserRecord::into_domain)
            .collect()
    }

    pub async fn find_auth_by_id(&self, id: UserId) -> StorageResult<Option<UserAuthRecord>> {
        self.find_record_by_id(&id).await?.map(UserRecord::into_auth).transpose()
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        self.find_record(UserColumn::Email.eq(email).into())
            .await?
            .map(UserRecord::into_domain)
            .transpose()
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<UserAuthRecord>> {
        self.find_record(UserColumn::Username.eq(username).into())
            .await?
            .map(UserRecord::into_auth)
            .transpose()
    }

    pub async fn find_auth_by_email(&self, email: &str) -> StorageResult<Option<UserAuthRecord>> {
        self.find_record(UserColumn::Email.eq(email).into())
            .await?
            .map(UserRecord::into_auth)
            .transpose()
    }

    pub async fn record_login(&self, id: UserId) -> StorageResult<()> {
        let record = self.find_record_by_id(&id).await?.ok_or(StorageError::NotFound)?;
        let mut active: UserActiveModel = record.into();
        active.last_login_at = Set(Some(time::OffsetDateTime::now_utc()));
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        Ok(())
    }

    pub async fn list(&self, page: PageRequest, filters: UserListFilters) -> StorageResult<Page<User>> {
        self.list_slice(
            PageSliceRequest {
                offset: (page.page - PAGE_INDEX_OFFSET) * page.page_size,
                limit: page.page_size,
                page: page.page,
                page_size: page.page_size,
            },
            filters,
        )
        .await
    }

    pub async fn list_slice(&self, request: PageSliceRequest, filters: UserListFilters) -> StorageResult<Page<User>> {
        let query = filtered_users(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_asc(UserColumn::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        let items = items.into_iter().map(UserRecord::into_domain).collect::<StorageResult<Vec<_>>>()?;
        Ok(Page {
            items,
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn find_record_by_id(&self, id: &UserId) -> StorageResult<Option<UserRecord>> {
        self.find_record(UserColumn::Id.eq(id.0.as_str()).into()).await
    }

    async fn find_record_by_id_in_tx(&self, id: &UserId, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<UserRecord>> {
        active_users()
            .filter(UserColumn::Id.eq(id.0.as_str()))
            .one(tx)
            .await
            .map_err(StorageError::from)
    }

    async fn find_record(&self, filter: Condition) -> StorageResult<Option<UserRecord>> {
        active_users().filter(filter).one(self.database.connection()).await.map_err(StorageError::from)
    }

    async fn find_reset_token_in_tx(&self, token_hash: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<PasswordResetTokenRecord>> {
        password_reset_tokens::Entity::find()
            .filter(password_reset_tokens::Column::TokenHash.eq(token_hash))
            .one(tx)
            .await
            .map(|record| record.map(password_reset_token_record))
            .map_err(StorageError::from)
    }
}

async fn ensure_role_exists(db: &DatabaseConnection, role: &str) -> StorageResult<()> {
    let exists = role_records::Entity::find_by_id(role.to_owned()).one(db).await?.is_some();
    if exists {
        return Ok(());
    }
    Err(StorageError::Conflict(format!("role does not exist: {role}")))
}
