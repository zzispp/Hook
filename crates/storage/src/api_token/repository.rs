use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use types::{
    api_token::{ApiToken, ApiTokenListRequest, ApiTokenType},
    model::PatchField,
};

use crate::{Database, StorageError, StorageResult, json};

use super::{
    ApiTokenRecordInput, ApiTokenRecordPatch, ApiTokenUsageRecord,
    record::api_tokens::{ActiveModel as ApiTokenActiveModel, model_access_mode_value, token_type_value},
    record::{ApiTokenRecord, api_tokens},
    usage,
};

#[derive(Clone)]
pub struct ApiTokenStore {
    database: Database,
}

impl ApiTokenStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create_token(&self, input: ApiTokenRecordInput) -> StorageResult<ApiToken> {
        token_active_model(self.database.next_id(), input)?
            .insert(self.database.connection())
            .await?
            .into_domain()
    }

    pub async fn update_token(&self, user_id: &str, id: &str, input: ApiTokenRecordPatch) -> StorageResult<ApiToken> {
        let record = self.find_user_token_record(user_id, id).await?.ok_or(StorageError::NotFound)?;
        let mut active: ApiTokenActiveModel = record.into();
        apply_token_patch(&mut active, input)?;
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.find_user_token(user_id, id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_any_token(&self, id: &str, input: ApiTokenRecordPatch) -> StorageResult<ApiToken> {
        let record = self.find_token_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: ApiTokenActiveModel = record.into();
        apply_token_patch(&mut active, input)?;
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.find_token(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_token(&self, user_id: &str, id: &str) -> StorageResult<()> {
        let record = self.find_user_token_record(user_id, id).await?.ok_or(StorageError::NotFound)?;
        let active: ApiTokenActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn delete_any_token(&self, id: &str) -> StorageResult<()> {
        let record = self.find_token_record(id).await?.ok_or(StorageError::NotFound)?;
        let active: ApiTokenActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_user_token(&self, user_id: &str, id: &str) -> StorageResult<Option<ApiToken>> {
        match self.find_user_token_record(user_id, id).await? {
            Some(record) => record.into_domain().map(Some),
            None => Ok(None),
        }
    }

    pub async fn find_token(&self, id: &str) -> StorageResult<Option<ApiToken>> {
        match self.find_token_record(id).await? {
            Some(record) => record.into_domain().map(Some),
            None => Ok(None),
        }
    }

    pub async fn find_by_hash(&self, token_hash: &str) -> StorageResult<Option<ApiToken>> {
        match api_tokens::Entity::find()
            .filter(api_tokens::Column::TokenHash.eq(token_hash))
            .one(self.database.connection())
            .await?
        {
            Some(record) => record.into_domain().map(Some),
            None => Ok(None),
        }
    }

    pub async fn list_user_tokens(&self, user_id: &str, request: ApiTokenListRequest) -> StorageResult<types::api_token::ApiTokenListResponse> {
        list_tokens(self.database.connection(), filtered_user_tokens(user_id, request.clone()), request).await
    }

    pub async fn list_admin_tokens(&self, request: ApiTokenListRequest) -> StorageResult<types::api_token::ApiTokenListResponse> {
        list_tokens(self.database.connection(), filtered_admin_tokens(request.clone()), request).await
    }

    pub async fn record_usage(&self, input: ApiTokenUsageRecord) -> StorageResult<()> {
        usage::record_usage(self.database.connection(), &input).await
    }

    pub async fn record_usage_batch(&self, inputs: &[ApiTokenUsageRecord]) -> StorageResult<()> {
        usage::record_usage_batch(self.database.connection(), inputs).await
    }

    pub async fn record_usage_batch_once(&self, batch_id: &str, inputs: &[ApiTokenUsageRecord]) -> StorageResult<bool> {
        usage::record_usage_batch_once(self.database.connection(), batch_id, inputs).await
    }

    pub async fn delete_expired_tokens(&self) -> StorageResult<u64> {
        let result = api_tokens::Entity::delete_many()
            .filter(api_tokens::Column::ExpiresAt.is_not_null())
            .filter(api_tokens::Column::ExpiresAt.lt(time::OffsetDateTime::now_utc()))
            .exec(self.database.connection())
            .await?;
        Ok(result.rows_affected)
    }

    async fn find_user_token_record(&self, user_id: &str, id: &str) -> StorageResult<Option<ApiTokenRecord>> {
        api_tokens::Entity::find_by_id(id.to_owned())
            .filter(api_tokens::Column::UserId.eq(user_id))
            .filter(api_tokens::Column::TokenType.eq(token_type_value(ApiTokenType::User)))
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn find_token_record(&self, id: &str) -> StorageResult<Option<ApiTokenRecord>> {
        api_tokens::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }
}

fn filtered_user_tokens(user_id: &str, request: ApiTokenListRequest) -> sea_orm::Select<api_tokens::Entity> {
    let query = api_tokens::Entity::find()
        .filter(api_tokens::Column::UserId.eq(user_id))
        .filter(api_tokens::Column::TokenType.eq(token_type_value(ApiTokenType::User)));
    apply_token_filters(query, request)
}

fn filtered_admin_tokens(request: ApiTokenListRequest) -> sea_orm::Select<api_tokens::Entity> {
    apply_token_filters(api_tokens::Entity::find(), request)
}

fn apply_token_filters(mut query: sea_orm::Select<api_tokens::Entity>, request: ApiTokenListRequest) -> sea_orm::Select<api_tokens::Entity> {
    if let Some(is_active) = request.is_active {
        query = query.filter(api_tokens::Column::IsActive.eq(is_active));
    }
    if let Some(token_type) = request.token_type {
        query = query.filter(api_tokens::Column::TokenType.eq(token_type_value(token_type)));
    }
    if let Some(user_id) = request.user_id {
        query = query.filter(api_tokens::Column::UserId.eq(user_id));
    }
    match request.search {
        Some(search) if !search.is_empty() => query.filter(token_search_condition(&search)),
        _ => query,
    }
}

fn token_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(api_tokens::Column::Name.contains(search))
        .add(api_tokens::Column::TokenPrefix.contains(search))
        .add(api_tokens::Column::GroupCode.contains(search))
}

async fn list_tokens(
    db: &DatabaseConnection,
    query: sea_orm::Select<api_tokens::Entity>,
    request: ApiTokenListRequest,
) -> StorageResult<types::api_token::ApiTokenListResponse> {
    let records = query.order_by_desc(api_tokens::Column::CreatedAt).all(db).await?;
    let total = records.len() as u64;
    let tokens = records
        .into_iter()
        .skip(request.skip as usize)
        .take(request.limit as usize)
        .map(ApiTokenRecord::into_domain)
        .collect::<StorageResult<Vec<ApiToken>>>()?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(types::api_token::ApiTokenListResponse { tokens, total })
}

fn token_active_model(id: String, input: ApiTokenRecordInput) -> StorageResult<ApiTokenActiveModel> {
    let now = time::OffsetDateTime::now_utc();
    Ok(ApiTokenActiveModel {
        id: Set(id),
        user_id: Set(input.user_id),
        token_type: Set(token_type_value(input.token_type).into()),
        name: Set(input.name),
        token_value: Set(input.token_value),
        token_hash: Set(input.token_hash),
        token_prefix: Set(input.token_prefix),
        group_code: Set(input.group_code),
        expires_at: Set(input.expires_at),
        model_access_mode: Set(model_access_mode_value(input.model_access_mode).into()),
        allowed_model_ids: Set(json::encode_required(&input.allowed_model_ids)?),
        rate_limit_rpm: Set(input.rate_limit_rpm),
        quota_limit: Set(input.quota_limit),
        used_quota: Set(Decimal::ZERO),
        request_count: Set(0),
        is_active: Set(true),
        last_used_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    })
}

fn apply_token_patch(active: &mut ApiTokenActiveModel, input: ApiTokenRecordPatch) -> StorageResult<()> {
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    if let Some(group_code) = input.group_code {
        active.group_code = Set(group_code);
    }
    apply_optional_time(&mut active.expires_at, input.expires_at);
    if let Some(mode) = input.model_access_mode {
        active.model_access_mode = Set(model_access_mode_value(mode).into());
    }
    apply_allowed_models(active, input.allowed_model_ids)?;
    apply_optional_i64(&mut active.rate_limit_rpm, input.rate_limit_rpm);
    apply_optional_decimal(&mut active.quota_limit, input.quota_limit);
    if let Some(is_active) = input.is_active {
        active.is_active = Set(is_active);
    }
    Ok(())
}

fn apply_allowed_models(active: &mut ApiTokenActiveModel, patch: PatchField<Vec<String>>) -> StorageResult<()> {
    if let PatchField::Value(value) = patch {
        active.allowed_model_ids = Set(json::encode_required(&value)?);
    }
    Ok(())
}

fn apply_optional_time(field: &mut sea_orm::ActiveValue<Option<time::OffsetDateTime>>, patch: PatchField<time::OffsetDateTime>) {
    match patch {
        PatchField::Value(value) => *field = Set(Some(value)),
        PatchField::Null => *field = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_optional_i64(field: &mut sea_orm::ActiveValue<Option<i64>>, patch: PatchField<i64>) {
    match patch {
        PatchField::Value(value) => *field = Set(Some(value)),
        PatchField::Null => *field = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_optional_decimal(field: &mut sea_orm::ActiveValue<Option<Decimal>>, patch: PatchField<Decimal>) {
    match patch {
        PatchField::Value(value) => *field = Set(Some(value)),
        PatchField::Null => *field = Set(None),
        PatchField::Missing => {}
    }
}
