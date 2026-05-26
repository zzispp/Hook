use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use time::format_description::well_known::Rfc3339;
use types::provider::{ProviderModelCost, ProviderModelCostMode};

use crate::{StorageError, StorageResult};

use super::{ProviderModelCostRecordInput, record::provider_model_costs, repository::ProviderStore};

pub async fn list_model_costs(store: &ProviderStore, provider_id: &str) -> StorageResult<Vec<ProviderModelCost>> {
    let records = provider_model_costs::Entity::find()
        .filter(provider_model_costs::Column::ProviderId.eq(provider_id))
        .order_by_asc(provider_model_costs::Column::KeyId)
        .order_by_asc(provider_model_costs::Column::ProviderModelId)
        .all(store.connection())
        .await?;
    records.into_iter().map(model_cost_response).collect()
}

pub async fn upsert_model_costs(store: &ProviderStore, inputs: Vec<ProviderModelCostRecordInput>) -> StorageResult<Vec<ProviderModelCost>> {
    let mut costs = Vec::with_capacity(inputs.len());
    for input in inputs {
        costs.push(upsert_model_cost(store, input).await?);
    }
    Ok(costs)
}

pub async fn delete_model_cost(store: &ProviderStore, provider_id: &str, key_id: &str, provider_model_id: &str) -> StorageResult<()> {
    let record = provider_model_cost_record(store, provider_id, key_id, provider_model_id).await?;
    let active: provider_model_costs::ActiveModel = record.into();
    active.delete(store.connection()).await?;
    Ok(())
}

pub async fn find_model_cost(store: &ProviderStore, key_id: &str, provider_model_id: &str) -> StorageResult<Option<ProviderModelCost>> {
    provider_model_costs::Entity::find()
        .filter(provider_model_costs::Column::KeyId.eq(key_id))
        .filter(provider_model_costs::Column::ProviderModelId.eq(provider_model_id))
        .one(store.connection())
        .await?
        .map(model_cost_response)
        .transpose()
}

async fn upsert_model_cost(store: &ProviderStore, input: ProviderModelCostRecordInput) -> StorageResult<ProviderModelCost> {
    let now = time::OffsetDateTime::now_utc();
    let existing = provider_model_costs::Entity::find()
        .filter(provider_model_costs::Column::KeyId.eq(&input.key_id))
        .filter(provider_model_costs::Column::ProviderModelId.eq(&input.provider_model_id))
        .one(store.connection())
        .await?;
    let record = match existing {
        Some(record) => update_model_cost(record, input, now).update(store.connection()).await?,
        None => create_model_cost(store, input, now).insert(store.connection()).await?,
    };
    model_cost_response(record)
}

fn create_model_cost(store: &ProviderStore, input: ProviderModelCostRecordInput, now: time::OffsetDateTime) -> provider_model_costs::ActiveModel {
    provider_model_cost_active_model(store.next_id(), input, now, now)
}

fn update_model_cost(record: provider_model_costs::Model, input: ProviderModelCostRecordInput, now: time::OffsetDateTime) -> provider_model_costs::ActiveModel {
    provider_model_cost_active_model(record.id, input, record.created_at, now)
}

fn provider_model_cost_active_model(
    id: String,
    input: ProviderModelCostRecordInput,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
) -> provider_model_costs::ActiveModel {
    provider_model_costs::ActiveModel {
        id: Set(id),
        provider_id: Set(input.provider_id),
        key_id: Set(input.key_id),
        provider_model_id: Set(input.provider_model_id),
        cost_mode: Set(cost_mode_value(&input.cost_mode).to_owned()),
        price_per_request: Set(input.price_per_request),
        input_price_per_million: Set(input.input_price_per_million),
        output_price_per_million: Set(input.output_price_per_million),
        cache_creation_price_per_million: Set(input.cache_creation_price_per_million),
        cache_read_price_per_million: Set(input.cache_read_price_per_million),
        created_at: Set(created_at),
        updated_at: Set(updated_at),
    }
}

async fn provider_model_cost_record(
    store: &ProviderStore,
    provider_id: &str,
    key_id: &str,
    provider_model_id: &str,
) -> StorageResult<provider_model_costs::Model> {
    let record = provider_model_costs::Entity::find()
        .filter(provider_model_costs::Column::ProviderId.eq(provider_id))
        .filter(provider_model_costs::Column::KeyId.eq(key_id))
        .filter(provider_model_costs::Column::ProviderModelId.eq(provider_model_id))
        .one(store.connection())
        .await?;
    record.ok_or(StorageError::NotFound)
}

fn model_cost_response(record: provider_model_costs::Model) -> StorageResult<ProviderModelCost> {
    Ok(ProviderModelCost {
        id: record.id,
        provider_id: record.provider_id,
        key_id: record.key_id,
        provider_model_id: record.provider_model_id,
        cost_mode: parse_cost_mode(&record.cost_mode)?,
        price_per_request: record.price_per_request,
        input_price_per_million: record.input_price_per_million,
        output_price_per_million: record.output_price_per_million,
        cache_creation_price_per_million: record.cache_creation_price_per_million,
        cache_read_price_per_million: record.cache_read_price_per_million,
        created_at: format_timestamp(record.created_at),
        updated_at: format_timestamp(record.updated_at),
    })
}

fn cost_mode_value(mode: &ProviderModelCostMode) -> &'static str {
    match mode {
        ProviderModelCostMode::PerRequest => "per_request",
        ProviderModelCostMode::PerToken => "per_token",
    }
}

fn parse_cost_mode(value: &str) -> StorageResult<ProviderModelCostMode> {
    match value {
        "per_request" => Ok(ProviderModelCostMode::PerRequest),
        "per_token" => Ok(ProviderModelCostMode::PerToken),
        other => Err(StorageError::Database(format!("invalid provider model cost mode: {other}"))),
    }
}

fn format_timestamp(value: sea_orm::prelude::TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("provider model cost timestamp must format as RFC3339")
}
