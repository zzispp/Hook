use constants::pagination::PAGE_INDEX_OFFSET;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{User, UserId, UserListFilters},
    user_group::{UserGroup, UserGroupListRequest, UserGroupPageResponse, UserGroupResponse},
};

use crate::{
    Database, StorageError, StorageResult, json,
    rbac::role_records,
    user::UserColumn,
    user::password_reset_tokens::{self, ActiveModel as PasswordResetTokenActiveModel},
    user::record::ActiveModel as UserActiveModel,
    user::user_groups::ActiveModel as UserGroupActiveModel,
};

use super::{
    PasswordResetTokenRecord, PasswordResetTokenRecordInput, UserAuthRecord, UserGroupRecord, UserGroupRecordInput, UserGroupRecordPatch, UserRecord,
    UserRecordInput,
    query::{active_users, filtered_users},
    tokens::{password_reset_token_active_model, password_reset_token_record},
    user_groups,
    user_mutations::{delete_user_api_tokens, required_password_hash, set_wallet_limit_mode},
};

#[derive(Clone)]
pub struct UserStore {
    pub(super) database: Database,
}

#[derive(Clone)]
pub struct UserGroupStore {
    database: Database,
}

impl UserStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, user: UserRecordInput) -> StorageResult<User> {
        ensure_role_exists(self.database.connection(), &user.role).await?;
        ensure_active_user_group_exists(self.database.connection(), &user.group_code).await?;
        let now = time::OffsetDateTime::now_utc();
        UserActiveModel {
            id: Set(self.database.next_id()),
            username: Set(user.username),
            password_hash: Set(required_password_hash(user.password_hash)?),
            email: Set(user.email),
            role: Set(user.role),
            group_code: Set(user.group_code),
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
        ensure_active_user_group_exists(self.database.connection(), &user.group_code).await?;
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
        active.group_code = Set(user.group_code);
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

impl UserGroupStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create_group(&self, input: UserGroupRecordInput) -> StorageResult<UserGroup> {
        let now = time::OffsetDateTime::now_utc();
        UserGroupActiveModel {
            id: Set(self.database.next_id()),
            code: Set(input.code),
            name: Set(input.name),
            description: Set(input.description),
            is_active: Set(input.is_active),
            is_system: Set(input.is_system),
            sort_order: Set(input.sort_order),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await
        .map(Into::into)
        .map_err(StorageError::from)
    }

    pub async fn update_group(&self, code: &str, input: UserGroupRecordPatch) -> StorageResult<UserGroup> {
        let record = self.find_group_record(code).await?.ok_or(StorageError::NotFound)?;
        let mut active: UserGroupActiveModel = record.into();
        apply_user_group_patch(&mut active, input);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await.map(Into::into).map_err(StorageError::from)
    }

    pub async fn delete_group(&self, code: &str) -> StorageResult<()> {
        let record = self.find_group_record(code).await?.ok_or(StorageError::NotFound)?;
        let active: UserGroupActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_group(&self, code: &str) -> StorageResult<Option<UserGroup>> {
        self.find_group_record(code).await.map(|record| record.map(Into::into))
    }

    pub async fn list_groups(&self, request: UserGroupListRequest) -> StorageResult<UserGroupPageResponse> {
        let query = filtered_user_groups(request.filters);
        let total = query.clone().count(self.database.connection()).await?;
        let records = query
            .order_by_asc(user_groups::Column::SortOrder)
            .order_by_asc(user_groups::Column::Code)
            .limit(request.page.page_size)
            .offset((request.page.page - PAGE_INDEX_OFFSET) * request.page.page_size)
            .all(self.database.connection())
            .await?;
        Ok(UserGroupPageResponse {
            items: records.into_iter().map(UserGroup::from).map(UserGroupResponse::from).collect(),
            total,
            page: request.page.page,
            page_size: request.page.page_size,
        })
    }

    pub async fn group_has_users(&self, code: &str) -> StorageResult<bool> {
        active_users()
            .filter(UserColumn::GroupCode.eq(code))
            .one(self.database.connection())
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    pub async fn active_group_exists(&self, code: &str) -> StorageResult<bool> {
        user_groups::Entity::find()
            .filter(user_groups::Column::Code.eq(code))
            .filter(user_groups::Column::IsActive.eq(true))
            .one(self.database.connection())
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    async fn find_group_record(&self, code: &str) -> StorageResult<Option<UserGroupRecord>> {
        user_groups::Entity::find()
            .filter(user_groups::Column::Code.eq(code))
            .one(self.database.connection())
            .await
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

async fn ensure_active_user_group_exists(db: &DatabaseConnection, code: &str) -> StorageResult<()> {
    let exists = user_groups::Entity::find()
        .filter(user_groups::Column::Code.eq(code))
        .filter(user_groups::Column::IsActive.eq(true))
        .one(db)
        .await?
        .is_some();
    if exists {
        return Ok(());
    }
    Err(StorageError::Conflict(format!("active user group does not exist: {code}")))
}

fn filtered_user_groups(filters: types::user_group::UserGroupFilters) -> sea_orm::Select<user_groups::Entity> {
    let mut query = user_groups::Entity::find();
    if let Some(is_active) = filters.is_active {
        query = query.filter(user_groups::Column::IsActive.eq(is_active));
    }
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(user_group_search_condition(&search)),
        _ => query,
    }
}

fn user_group_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(user_groups::Column::Code.contains(search))
        .add(user_groups::Column::Name.contains(search))
        .add(user_groups::Column::Description.contains(search))
}

fn apply_user_group_patch(active: &mut UserGroupActiveModel, input: UserGroupRecordPatch) {
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    if let Some(description) = input.description {
        active.description = Set(nonempty_optional(description));
    }
    if let Some(is_active) = input.is_active {
        active.is_active = Set(is_active);
    }
    if let Some(sort_order) = input.sort_order {
        active.sort_order = Set(sort_order);
    }
}

fn nonempty_optional(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    if value.is_empty() { None } else { Some(value) }
}
