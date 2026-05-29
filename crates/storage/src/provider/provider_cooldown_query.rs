use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use time::format_description::well_known::Rfc3339;
use types::provider::{ProviderCooldown, ProviderCooldownListRequest, ProviderCooldownListResponse};

use crate::{StorageError, StorageResult};

use super::{
    ProviderCooldownEventRecordInput, ProviderCooldownRecordInput,
    record::{provider_cooldown_events, provider_cooldowns},
    repository::ProviderStore,
};

pub async fn upsert_provider_cooldown(store: &ProviderStore, input: ProviderCooldownRecordInput) -> StorageResult<ProviderCooldown> {
    let now = time::OffsetDateTime::now_utc();
    let record = match provider_cooldowns::Entity::find_by_id(input.provider_id.clone())
        .one(store.connection())
        .await?
    {
        Some(record) => update_provider_cooldown_record(store, record, input, now).await?,
        None => insert_provider_cooldown_record(store, input, now).await?,
    };
    Ok(provider_cooldown(record))
}

pub async fn create_provider_cooldown_event(store: &ProviderStore, input: ProviderCooldownEventRecordInput) -> StorageResult<()> {
    let now = time::OffsetDateTime::now_utc();
    let active = provider_cooldown_events::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(input.provider_id),
        provider_name_snapshot: Set(input.provider_name_snapshot),
        status_code: Set(input.status_code),
        observed_count: Set(input.observed_count),
        threshold_count: Set(input.threshold_count),
        window_seconds: Set(input.window_seconds),
        cooldown_seconds: Set(input.cooldown_seconds),
        triggered_at: Set(input.triggered_at),
        cooldown_until: Set(input.cooldown_until),
        request_id: Set(input.request_id),
        candidate_index: Set(input.candidate_index),
        retry_index: Set(input.retry_index),
        endpoint_id: Set(input.endpoint_id),
        endpoint_name_snapshot: Set(input.endpoint_name_snapshot),
        key_id: Set(input.key_id),
        key_name_snapshot: Set(input.key_name_snapshot),
        error_type: Set(input.error_type),
        error_message: Set(input.error_message),
        error_code: Set(input.error_code),
        error_param: Set(input.error_param),
        created_at: Set(now),
    };
    active.insert(store.connection()).await?;
    Ok(())
}

async fn insert_provider_cooldown_record(
    store: &ProviderStore,
    input: ProviderCooldownRecordInput,
    now: time::OffsetDateTime,
) -> StorageResult<provider_cooldowns::Model> {
    let mut active = provider_cooldowns::ActiveModel {
        provider_id: Set(input.provider_id.clone()),
        created_at: Set(now),
        ..Default::default()
    };
    apply_record_input(&mut active, input, now);
    active.insert(store.connection()).await.map_err(Into::into)
}

async fn update_provider_cooldown_record(
    store: &ProviderStore,
    record: provider_cooldowns::Model,
    input: ProviderCooldownRecordInput,
    now: time::OffsetDateTime,
) -> StorageResult<provider_cooldowns::Model> {
    let mut active: provider_cooldowns::ActiveModel = record.into();
    apply_record_input(&mut active, input, now);
    active.update(store.connection()).await.map_err(Into::into)
}

fn apply_record_input(active: &mut provider_cooldowns::ActiveModel, input: ProviderCooldownRecordInput, now: time::OffsetDateTime) {
    active.provider_name_snapshot = Set(input.provider_name_snapshot);
    active.status_code = Set(input.status_code);
    active.observed_count = Set(input.observed_count);
    active.threshold_count = Set(input.threshold_count);
    active.window_seconds = Set(input.window_seconds);
    active.cooldown_seconds = Set(input.cooldown_seconds);
    active.triggered_at = Set(input.triggered_at);
    active.cooldown_until = Set(input.cooldown_until);
    active.released_at = Set(None);
    active.request_id = Set(input.request_id);
    active.candidate_index = Set(input.candidate_index);
    active.retry_index = Set(input.retry_index);
    active.endpoint_id = Set(input.endpoint_id);
    active.endpoint_name_snapshot = Set(input.endpoint_name_snapshot);
    active.key_id = Set(input.key_id);
    active.key_name_snapshot = Set(input.key_name_snapshot);
    active.error_type = Set(input.error_type);
    active.error_message = Set(input.error_message);
    active.error_code = Set(input.error_code);
    active.error_param = Set(input.error_param);
    active.updated_at = Set(now);
}

pub async fn list_active_provider_cooldowns(store: &ProviderStore, request: ProviderCooldownListRequest) -> StorageResult<ProviderCooldownListResponse> {
    let now = time::OffsetDateTime::now_utc();
    let mut query = provider_cooldowns::Entity::find()
        .filter(provider_cooldowns::Column::ReleasedAt.is_null())
        .filter(provider_cooldowns::Column::CooldownUntil.gt(now))
        .order_by_asc(provider_cooldowns::Column::CooldownUntil);
    query = apply_filters(query, &request);
    let total = query.clone().count(store.connection()).await?;
    let records = query.offset(request.skip).limit(request.limit).all(store.connection()).await?;
    Ok(ProviderCooldownListResponse {
        cooldowns: records.into_iter().map(provider_cooldown).collect(),
        total,
    })
}

pub async fn release_provider_cooldown(store: &ProviderStore, provider_id: &str) -> StorageResult<ProviderCooldown> {
    let now = time::OffsetDateTime::now_utc();
    let Some(record) = provider_cooldowns::Entity::find_by_id(provider_id.to_owned())
        .filter(provider_cooldowns::Column::ReleasedAt.is_null())
        .filter(provider_cooldowns::Column::CooldownUntil.gt(now))
        .one(store.connection())
        .await?
    else {
        return Err(StorageError::NotFound);
    };
    let mut active: provider_cooldowns::ActiveModel = record.into();
    active.released_at = Set(Some(now));
    active.updated_at = Set(now);
    let record = active.update(store.connection()).await?;
    Ok(provider_cooldown(record))
}

pub async fn active_provider_cooldowns_for_restore(store: &ProviderStore) -> StorageResult<Vec<ProviderCooldown>> {
    let now = time::OffsetDateTime::now_utc();
    let records = provider_cooldowns::Entity::find()
        .filter(provider_cooldowns::Column::ReleasedAt.is_null())
        .filter(provider_cooldowns::Column::CooldownUntil.gt(now))
        .all(store.connection())
        .await?;
    Ok(records.into_iter().map(provider_cooldown).collect())
}

fn apply_filters(mut query: sea_orm::Select<provider_cooldowns::Entity>, request: &ProviderCooldownListRequest) -> sea_orm::Select<provider_cooldowns::Entity> {
    if let Some(status_code) = request.status_code {
        query = query.filter(provider_cooldowns::Column::StatusCode.eq(status_code));
    }
    if let Some(search) = request.search.as_deref().filter(|value| !value.is_empty()) {
        query = query.filter(provider_cooldowns::Column::ProviderNameSnapshot.contains(search));
    }
    query
}

fn provider_cooldown(record: provider_cooldowns::Model) -> ProviderCooldown {
    ProviderCooldown {
        provider_id: record.provider_id,
        provider_name: record.provider_name_snapshot,
        status_code: record.status_code,
        observed_count: record.observed_count,
        threshold_count: record.threshold_count,
        window_seconds: record.window_seconds,
        cooldown_seconds: record.cooldown_seconds,
        triggered_at: format_timestamp(record.triggered_at),
        cooldown_until: format_timestamp(record.cooldown_until),
        released_at: record.released_at.map(format_timestamp),
        request_id: record.request_id,
        candidate_index: record.candidate_index,
        retry_index: record.retry_index,
        endpoint_id: record.endpoint_id,
        endpoint_name: record.endpoint_name_snapshot,
        key_id: record.key_id,
        key_name: record.key_name_snapshot,
        error_type: record.error_type,
        error_message: record.error_message,
        error_code: record.error_code,
        error_param: record.error_param,
        created_at: format_timestamp(record.created_at),
        updated_at: format_timestamp(record.updated_at),
    }
}

fn format_timestamp(value: sea_orm::entity::prelude::TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("provider cooldown timestamp must format as RFC3339")
}
