use std::collections::BTreeMap;

use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set, TransactionTrait};
use types::provider::{ProviderModelCostMode, ProviderOrigin};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderQuickImportApiKeyRecordInput, ProviderQuickImportEndpointRecordInput, ProviderQuickImportModelCostRecordInput, ProviderQuickImportModelRecordInput,
    ProviderQuickImportRecordInput, ProviderQuickImportRecordOutput, ProviderStore,
    provider_model_cost_query::model_cost_response,
    record::{provider_api_keys, provider_endpoints, provider_model_costs, provider_models},
    repository_helpers::{provider_active_model, provider_model_response},
};

pub async fn create_quick_import(store: &ProviderStore, input: ProviderQuickImportRecordInput) -> StorageResult<ProviderQuickImportRecordOutput> {
    let tx = store.connection().begin().await?;
    let provider_id = store.next_id();
    let provider = provider_active_model(provider_id.clone(), quick_import_provider(input.provider))
        .insert(&tx)
        .await?
        .into();
    let endpoints = insert_endpoints(store, &tx, &provider_id, input.endpoints).await?;
    let (model_bindings, model_ids) = insert_models(store, &tx, &provider_id, input.model_bindings).await?;
    let (api_keys, key_ids) = insert_keys(store, &tx, &provider_id, input.api_keys).await?;
    let model_costs = insert_costs(store, &tx, &provider_id, model_ids, key_ids, input.model_costs).await?;
    tx.commit().await?;
    Ok(ProviderQuickImportRecordOutput {
        provider,
        endpoints,
        api_keys,
        model_bindings,
        model_costs,
    })
}

fn quick_import_provider(input: super::ProviderRecordInput) -> super::ProviderRecordInput {
    super::ProviderRecordInput {
        provider_origin: ProviderOrigin::QuickImport,
        ..input
    }
}

async fn insert_endpoints(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    inputs: Vec<ProviderQuickImportEndpointRecordInput>,
) -> StorageResult<Vec<types::provider::ProviderEndpoint>> {
    let mut output = Vec::with_capacity(inputs.len());
    for input in inputs {
        let active = endpoint_active_model(store, provider_id, input)?;
        output.push(active.insert(tx).await?.response()?);
    }
    Ok(output)
}

async fn insert_models(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    inputs: Vec<ProviderQuickImportModelRecordInput>,
) -> StorageResult<(Vec<types::provider::ProviderModelBinding>, BTreeMap<String, String>)> {
    let mut output = Vec::with_capacity(inputs.len());
    let mut ids = BTreeMap::new();
    for input in inputs {
        let global_model_id = input.global_model_id.clone();
        let record = model_active_model(store, provider_id, input)?.insert(tx).await?;
        ids.insert(global_model_id, record.id.clone());
        output.push(provider_model_response(record)?);
    }
    Ok((output, ids))
}

async fn insert_keys(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    inputs: Vec<ProviderQuickImportApiKeyRecordInput>,
) -> StorageResult<(Vec<types::provider::ProviderApiKey>, BTreeMap<String, String>)> {
    let mut output = Vec::with_capacity(inputs.len());
    let mut ids = BTreeMap::new();
    for input in inputs {
        let upstream_token_id = input.upstream_token_id.clone();
        let record = key_active_model(store, provider_id, input)?.insert(tx).await?;
        ids.insert(upstream_token_id, record.id.clone());
        output.push(record.response()?);
    }
    Ok((output, ids))
}

async fn insert_costs(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    model_ids: BTreeMap<String, String>,
    key_ids: BTreeMap<String, String>,
    inputs: Vec<ProviderQuickImportModelCostRecordInput>,
) -> StorageResult<Vec<types::provider::ProviderModelCost>> {
    let mut output = Vec::with_capacity(inputs.len());
    for input in inputs {
        let active = cost_active_model(store, provider_id, &model_ids, &key_ids, input)?;
        output.push(model_cost_response(active.insert(tx).await?)?);
    }
    Ok(output)
}

fn endpoint_active_model(
    store: &ProviderStore,
    provider_id: &str,
    input: ProviderQuickImportEndpointRecordInput,
) -> StorageResult<provider_endpoints::ActiveModel> {
    let now = time::OffsetDateTime::now_utc();
    Ok(provider_endpoints::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(provider_id.to_owned()),
        api_format: Set(input.api_format),
        base_url: Set(input.base_url),
        custom_path: Set(input.custom_path),
        max_retries: Set(input.max_retries),
        is_active: Set(input.is_active),
        format_acceptance_config: Set(json::encode_optional(&input.format_acceptance_config)?),
        header_rules: Set(json::encode_optional(&input.header_rules)?),
        body_rules: Set(json::encode_optional(&input.body_rules)?),
        created_at: Set(now),
        updated_at: Set(now),
    })
}

fn model_active_model(store: &ProviderStore, provider_id: &str, input: ProviderQuickImportModelRecordInput) -> StorageResult<provider_models::ActiveModel> {
    let now = time::OffsetDateTime::now_utc();
    Ok(provider_models::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(provider_id.to_owned()),
        global_model_id: Set(input.global_model_id),
        provider_model_name: Set(input.provider_model_name),
        provider_model_mappings: Set(json::encode_optional(&input.provider_model_mapping)?),
        is_active: Set(input.is_active),
        config: Set(json::encode_optional(&input.config)?),
        created_at: Set(now),
        updated_at: Set(now),
    })
}

fn key_active_model(store: &ProviderStore, provider_id: &str, input: ProviderQuickImportApiKeyRecordInput) -> StorageResult<provider_api_keys::ActiveModel> {
    let now = time::OffsetDateTime::now_utc();
    Ok(provider_api_keys::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(provider_id.to_owned()),
        name: Set(input.name),
        api_formats: Set(json::encode_required(&input.api_formats)?),
        allowed_model_ids: Set(json::encode_required(&input.allowed_model_ids)?),
        encrypted_api_key: Set(input.encrypted_api_key),
        note: Set(input.note),
        internal_priority: Set(input.internal_priority),
        global_priority_by_format: Set(json::encode_required(&input.global_priority_by_format)?),
        rpm_limit: Set(input.rpm_limit),
        learned_rpm_limit: Set(None),
        cache_ttl_minutes: Set(input.cache_ttl_minutes),
        max_probe_interval_minutes: Set(input.max_probe_interval_minutes),
        time_range_enabled: Set(input.time_range_enabled),
        time_range_start: Set(input.time_range_start),
        time_range_end: Set(input.time_range_end),
        health_by_format: Set(None),
        circuit_breaker_by_format: Set(None),
        is_active: Set(input.is_active),
        created_at: Set(now),
        updated_at: Set(now),
    })
}

fn cost_active_model(
    store: &ProviderStore,
    provider_id: &str,
    model_ids: &BTreeMap<String, String>,
    key_ids: &BTreeMap<String, String>,
    input: ProviderQuickImportModelCostRecordInput,
) -> StorageResult<provider_model_costs::ActiveModel> {
    let now = time::OffsetDateTime::now_utc();
    Ok(provider_model_costs::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(provider_id.to_owned()),
        key_id: Set(mapped_id(key_ids, &input.upstream_token_id, "upstream token")?),
        provider_model_id: Set(mapped_id(model_ids, &input.global_model_id, "global model")?),
        cost_mode: Set(cost_mode_value(&input.cost_mode).to_owned()),
        price_per_request: Set(input.price_per_request),
        input_price_per_million: Set(input.input_price_per_million),
        output_price_per_million: Set(input.output_price_per_million),
        cache_creation_price_per_million: Set(input.cache_creation_price_per_million),
        cache_read_price_per_million: Set(input.cache_read_price_per_million),
        created_at: Set(now),
        updated_at: Set(now),
    })
}

fn mapped_id(ids: &BTreeMap<String, String>, key: &str, label: &str) -> StorageResult<String> {
    ids.get(key)
        .cloned()
        .ok_or_else(|| StorageError::Database(format!("quick import {label} id is missing: {key}")))
}

fn cost_mode_value(mode: &ProviderModelCostMode) -> &'static str {
    match mode {
        ProviderModelCostMode::PerRequest => "per_request",
        ProviderModelCostMode::PerToken => "per_token",
    }
}
