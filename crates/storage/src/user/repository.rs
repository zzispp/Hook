use constants::pagination::PAGE_INDEX_OFFSET;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use std::collections::{BTreeMap, BTreeSet};

use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{IdentityProvider, User, UserId, UserIdentity, UserIdentityInput, UserListFilters},
    user_group::{UserGroup, UserGroupListRequest, UserGroupPageResponse, UserGroupResponse},
};

use crate::{
    Database, StorageError, StorageResult, json,
    rbac::role_records,
    user::UserColumn,
    user::identity_record::{ActiveModel as UserIdentityActiveModel, Entity as UserIdentityEntity},
    user::password_reset_tokens::{self, ActiveModel as PasswordResetTokenActiveModel},
    user::record::ActiveModel as UserActiveModel,
    user::user_group_memberships::ActiveModel as UserGroupMembershipActiveModel,
    user::user_groups::ActiveModel as UserGroupActiveModel,
};

use super::{
    PasswordResetTokenRecord, PasswordResetTokenRecordInput, UserAuthRecord, UserGroupRecord, UserGroupRecordInput, UserGroupRecordPatch, UserIdentityRecord,
    UserRecord, UserRecordInput,
    query::{active_users, filtered_users},
    tokens::{password_reset_token_active_model, password_reset_token_record},
    user_group_memberships, user_groups,
    user_mutations::{delete_user_api_tokens, set_wallet_limit_mode},
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
        ensure_active_user_groups_exist(self.database.connection(), &user.group_codes).await?;
        let now = time::OffsetDateTime::now_utc();
        let group_codes = user.group_codes.clone();
        let tx = self.database.connection().begin().await?;
        let user_id = self.database.next_id();
        let referrer_user_id = referrer_user_id(user.referrer_aff_code.as_deref(), &tx).await?;
        let referred_at = referrer_user_id.as_ref().map(|_| now);
        let record = UserActiveModel {
            id: Set(user_id.clone()),
            username: Set(user.username),
            password_hash: Set(user.password_hash),
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
            affiliate_code: Set(affiliate_code_from_id(&user_id)),
            referred_by_user_id: Set(referrer_user_id),
            referred_at: Set(referred_at),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&tx)
        .await
        .map_err(StorageError::from)?;
        replace_user_groups(&record.id, group_codes, self, &tx).await?;
        tx.commit().await?;
        self.find_by_id(UserId(record.id)).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: UserId, user: UserRecordInput) -> StorageResult<User> {
        ensure_role_exists(self.database.connection(), &user.role).await?;
        ensure_active_user_groups_exist(self.database.connection(), &user.group_codes).await?;
        let tx = self.database.connection().begin().await?;
        let record = self.find_record_by_id_in_tx(&id, &tx).await?.ok_or(StorageError::NotFound)?;
        let mut active: UserActiveModel = record.into();
        let quota_mode = user.quota_mode.clone();
        let group_codes = user.group_codes.clone();
        active.username = Set(user.username);
        if let Some(password_hash) = user.password_hash {
            active.password_hash = Set(Some(password_hash));
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
        replace_user_groups(&id.0, group_codes, self, &tx).await?;
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
        user_active.password_hash = Set(Some(password_hash.to_owned()));
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
        self.optional_user(self.find_record_by_id(&id).await?).await
    }

    pub async fn find_by_ids(&self, ids: &[String]) -> StorageResult<Vec<User>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let records = active_users()
            .filter(UserColumn::Id.is_in(ids.iter().cloned()))
            .all(self.database.connection())
            .await
            .map_err(StorageError::from)?;
        self.users_from_records(records).await
    }

    pub async fn find_auth_by_id(&self, id: UserId) -> StorageResult<Option<UserAuthRecord>> {
        self.optional_auth(self.find_record_by_id(&id).await?).await
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        self.optional_user(self.find_record(UserColumn::Email.eq(email).into()).await?).await
    }

    pub async fn find_by_affiliate_code(&self, affiliate_code: &str) -> StorageResult<Option<User>> {
        self.optional_user(self.find_record(UserColumn::AffiliateCode.eq(affiliate_code).into()).await?)
            .await
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<UserAuthRecord>> {
        self.optional_auth(self.find_record(UserColumn::Username.eq(username).into()).await?).await
    }

    pub async fn find_auth_by_email(&self, email: &str) -> StorageResult<Option<UserAuthRecord>> {
        self.optional_auth(self.find_record(UserColumn::Email.eq(email).into()).await?).await
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
            .order_by_desc(UserColumn::CreatedAt)
            .order_by_asc(UserColumn::Id)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        let items = self.users_from_records(items).await?;
        Ok(Page {
            items,
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    pub async fn create_identity(&self, input: UserIdentityInput) -> StorageResult<UserIdentity> {
        let now = time::OffsetDateTime::now_utc();
        UserIdentityActiveModel {
            id: Set(self.database.next_id()),
            user_id: Set(input.user_id),
            provider: Set(input.provider.as_str().to_owned()),
            provider_subject: Set(input.provider_subject),
            email: Set(input.email),
            email_verified: Set(input.email_verified),
            display_name: Set(input.display_name),
            avatar_url: Set(input.avatar_url),
            metadata_json: Set(input.metadata_json),
            created_at: Set(now),
            updated_at: Set(now),
            last_login_at: Set(None),
        }
        .insert(self.database.connection())
        .await
        .map_err(StorageError::from)?
        .into_domain()
    }

    pub async fn find_identity(&self, provider: IdentityProvider, subject: &str) -> StorageResult<Option<UserIdentity>> {
        UserIdentityEntity::find()
            .filter(super::identity_record::Column::Provider.eq(provider.as_str()))
            .filter(super::identity_record::Column::ProviderSubject.eq(subject))
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)?
            .map(UserIdentityRecord::into_domain)
            .transpose()
    }

    pub async fn list_identities_by_user_id(&self, user_id: &str) -> StorageResult<Vec<UserIdentity>> {
        UserIdentityEntity::find()
            .filter(super::identity_record::Column::UserId.eq(user_id))
            .all(self.database.connection())
            .await
            .map_err(StorageError::from)?
            .into_iter()
            .map(UserIdentityRecord::into_domain)
            .collect()
    }

    pub async fn list_identities_by_user_ids(&self, user_ids: &[String]) -> StorageResult<BTreeMap<String, Vec<UserIdentity>>> {
        if user_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let identities = UserIdentityEntity::find()
            .filter(super::identity_record::Column::UserId.is_in(user_ids.iter().cloned()))
            .all(self.database.connection())
            .await
            .map_err(StorageError::from)?;
        group_identities(identities)
    }

    pub async fn touch_identity_login(&self, identity_id: &str) -> StorageResult<()> {
        let record = UserIdentityEntity::find_by_id(identity_id.to_owned())
            .one(self.database.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        let mut active: UserIdentityActiveModel = record.into();
        let now = time::OffsetDateTime::now_utc();
        active.last_login_at = Set(Some(now));
        active.updated_at = Set(now);
        active.update(self.database.connection()).await?;
        Ok(())
    }

    pub async fn delete_identity(&self, identity_id: &str) -> StorageResult<()> {
        let result = UserIdentityEntity::delete_by_id(identity_id.to_owned())
            .exec(self.database.connection())
            .await?;
        if result.rows_affected == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
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

    async fn optional_user(&self, record: Option<UserRecord>) -> StorageResult<Option<User>> {
        match record {
            Some(record) => self.user_from_record(record).await.map(Some),
            None => Ok(None),
        }
    }

    async fn optional_auth(&self, record: Option<UserRecord>) -> StorageResult<Option<UserAuthRecord>> {
        match record {
            Some(record) => self.auth_from_record(record).await.map(Some),
            None => Ok(None),
        }
    }

    async fn user_from_record(&self, record: UserRecord) -> StorageResult<User> {
        let group_codes = group_codes_for_user(&record.id, self.database.connection()).await?;
        record.into_domain(group_codes)
    }

    async fn auth_from_record(&self, record: UserRecord) -> StorageResult<UserAuthRecord> {
        let group_codes = group_codes_for_user(&record.id, self.database.connection()).await?;
        record.into_auth(group_codes)
    }

    async fn users_from_records(&self, records: Vec<UserRecord>) -> StorageResult<Vec<User>> {
        let ids = records.iter().map(|record| record.id.clone()).collect::<Vec<_>>();
        let group_codes = group_codes_by_user_ids(&ids, self.database.connection()).await?;
        records
            .into_iter()
            .map(|record| {
                let codes = group_codes.get(&record.id).cloned().unwrap_or_default();
                record.into_domain(codes)
            })
            .collect()
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

fn group_identities(records: Vec<UserIdentityRecord>) -> StorageResult<BTreeMap<String, Vec<UserIdentity>>> {
    let mut grouped = BTreeMap::<String, Vec<UserIdentity>>::new();
    for identity in records {
        let identity = identity.into_domain()?;
        grouped.entry(identity.user_id.clone()).or_default().push(identity);
    }
    Ok(grouped)
}

async fn replace_user_groups(user_id: &str, group_codes: Vec<String>, store: &UserStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    user_group_memberships::Entity::delete_many()
        .filter(user_group_memberships::Column::UserId.eq(user_id))
        .exec(tx)
        .await?;
    insert_user_groups(user_id, group_codes, store, tx).await
}

async fn referrer_user_id(referrer_aff_code: Option<&str>, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<String>> {
    let Some(code) = referrer_aff_code.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let record = active_users()
        .filter(UserColumn::AffiliateCode.eq(code.trim()))
        .lock_exclusive()
        .one(tx)
        .await?;
    record
        .map(|user| Some(user.id))
        .ok_or_else(|| StorageError::Conflict("referrer affiliate code does not exist".into()))
}

fn affiliate_code_from_id(id: &str) -> String {
    id.chars().filter(|ch| *ch != '-').collect::<String>()
}

async fn insert_user_groups(user_id: &str, group_codes: Vec<String>, store: &UserStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let group_codes = unique_group_codes(group_codes);
    if group_codes.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = group_codes
        .into_iter()
        .map(|group_code| user_group_membership_active_model(store.database.next_id(), user_id, group_code, now));
    user_group_memberships::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

fn user_group_membership_active_model(id: String, user_id: &str, group_code: String, now: time::OffsetDateTime) -> UserGroupMembershipActiveModel {
    UserGroupMembershipActiveModel {
        id: Set(id),
        user_id: Set(user_id.to_owned()),
        user_group_code: Set(group_code),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

async fn group_codes_for_user(user_id: &str, db: &DatabaseConnection) -> StorageResult<Vec<String>> {
    let mut grouped = group_codes_by_user_ids(&[user_id.to_owned()], db).await?;
    Ok(grouped.remove(user_id).unwrap_or_default())
}

async fn group_codes_by_user_ids(user_ids: &[String], db: &DatabaseConnection) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if user_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = user_group_memberships::Entity::find()
        .filter(user_group_memberships::Column::UserId.is_in(user_ids.iter().cloned()))
        .all(db)
        .await?;
    let order = user_group_order(records.iter().map(|record| record.user_group_code.clone()).collect(), db).await?;
    Ok(group_memberships(records, &order))
}

async fn user_group_order(group_codes: BTreeSet<String>, db: &DatabaseConnection) -> StorageResult<BTreeMap<String, i64>> {
    if group_codes.is_empty() {
        return Ok(BTreeMap::new());
    }
    let groups = user_groups::Entity::find()
        .filter(user_groups::Column::Code.is_in(group_codes))
        .order_by_asc(user_groups::Column::SortOrder)
        .order_by_asc(user_groups::Column::Code)
        .all(db)
        .await?;
    Ok(groups.into_iter().enumerate().map(|(index, group)| (group.code, index as i64)).collect())
}

fn group_memberships(records: Vec<user_group_memberships::Model>, order: &BTreeMap<String, i64>) -> BTreeMap<String, Vec<String>> {
    let mut grouped = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        grouped.entry(record.user_id).or_default().push(record.user_group_code);
    }
    for group_codes in grouped.values_mut() {
        group_codes.sort_by_key(|code| (*order.get(code).unwrap_or(&i64::MAX), code.clone()));
    }
    grouped
}

fn unique_group_codes(group_codes: Vec<String>) -> Vec<String> {
    group_codes.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
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
        user_group_memberships::Entity::find()
            .filter(user_group_memberships::Column::UserGroupCode.eq(code))
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

async fn ensure_active_user_groups_exist(db: &DatabaseConnection, codes: &[String]) -> StorageResult<()> {
    let requested = codes.iter().cloned().collect::<BTreeSet<_>>();
    if requested.is_empty() {
        return Err(StorageError::Conflict("user must belong to at least one user group".into()));
    }
    let found = user_groups::Entity::find()
        .filter(user_groups::Column::Code.is_in(requested.iter().cloned()))
        .filter(user_groups::Column::IsActive.eq(true))
        .all(db)
        .await?
        .into_iter()
        .map(|group| group.code)
        .collect::<BTreeSet<_>>();
    if found == requested {
        return Ok(());
    }
    let missing = requested.difference(&found).next().cloned().unwrap_or_default();
    Err(StorageError::Conflict(format!("active user group does not exist: {missing}")))
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
