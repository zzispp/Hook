use std::collections::BTreeMap;

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set, TransactionTrait};
use types::provider::{ProviderModelCostMode, ProviderOrigin, ProviderQuickImportSyncStatus};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderQuickImportApiKeyRecordInput, ProviderQuickImportAppendRecordInput, ProviderQuickImportAppendRecordOutput, ProviderQuickImportEndpointRecordInput,
    ProviderQuickImportKeyModelRecordInput, ProviderQuickImportKeyReplacementRecordInput, ProviderQuickImportKeyReplacementRecordOutput,
    ProviderQuickImportModelCostRecordInput, ProviderQuickImportModelRecordInput, ProviderQuickImportRecordInput, ProviderQuickImportRecordOutput,
    ProviderQuickImportSourceRecordInput, ProviderStore,
    provider_model_cost_query::model_cost_response,
    record::{
        provider_api_keys, provider_endpoints, provider_model_costs, provider_models, provider_quick_import_key_models, provider_quick_import_keys,
        provider_quick_import_sources,
    },
    repository_helpers::{apply_provider_api_key_patch, provider_active_model, provider_model_response},
};

pub async fn create_quick_import(store: &ProviderStore, input: ProviderQuickImportRecordInput) -> StorageResult<ProviderQuickImportRecordOutput> {
    let tx = store.connection().begin().await?;
    let provider_id = store.next_id();
    let sync_keys = sync_key_inputs(&input.api_keys);
    let provider = provider_active_model(provider_id.clone(), quick_import_provider(input.provider))
        .insert(&tx)
        .await?
        .into();
    let endpoints = insert_endpoints(store, &tx, &provider_id, input.endpoints).await?;
    let (model_bindings, model_ids) = insert_models(store, &tx, &provider_id, input.model_bindings).await?;
    let (api_keys, key_ids) = insert_keys(store, &tx, &provider_id, input.api_keys).await?;
    let model_costs = insert_costs(store, &tx, &provider_id, model_ids, key_ids.clone(), input.model_costs).await?;
    if let Some(sync_source) = input.sync_source {
        insert_sync_metadata(store, &tx, &provider_id, sync_source, sync_keys, &key_ids).await?;
    }
    tx.commit().await?;
    Ok(ProviderQuickImportRecordOutput {
        provider,
        endpoints,
        api_keys,
        model_bindings,
        model_costs,
    })
}

pub async fn append_quick_import(store: &ProviderStore, input: ProviderQuickImportAppendRecordInput) -> StorageResult<ProviderQuickImportAppendRecordOutput> {
    let tx = store.connection().begin().await?;
    let sync_keys = sync_key_inputs(&input.api_keys);
    let endpoints = insert_endpoints(store, &tx, &input.provider_id, input.endpoints).await?;
    let (model_bindings, new_model_ids) = insert_models(store, &tx, &input.provider_id, input.model_bindings).await?;
    let model_ids = merged_model_ids(store, &tx, &input.provider_id, new_model_ids).await?;
    let (api_keys, key_ids) = insert_keys(store, &tx, &input.provider_id, input.api_keys).await?;
    let model_costs = insert_costs(store, &tx, &input.provider_id, model_ids, key_ids.clone(), input.model_costs).await?;
    for key in sync_keys {
        insert_sync_key(store, &tx, &input.provider_id, &input.source_id, key, &key_ids).await?;
    }
    tx.commit().await?;
    Ok(ProviderQuickImportAppendRecordOutput {
        endpoints,
        api_keys,
        model_bindings,
        model_costs,
    })
}

pub async fn replace_quick_import_key(
    store: &ProviderStore,
    input: ProviderQuickImportKeyReplacementRecordInput,
) -> StorageResult<ProviderQuickImportKeyReplacementRecordOutput> {
    let tx = store.connection().begin().await?;
    let model_inputs = input.model_bindings.clone();
    let (model_bindings, new_model_ids) = insert_models(store, &tx, &input.provider_id, model_inputs).await?;
    let model_ids = merged_model_ids(store, &tx, &input.provider_id, new_model_ids).await?;
    delete_key_model_mappings(&tx, &input.provider_id, &input.key_id).await?;
    insert_replacement_model_mappings(store, &tx, &input, &input.key_id).await?;
    let api_key = update_replacement_api_key(&tx, &input).await?;
    update_replacement_sync_key(&tx, &input).await?;
    delete_key_costs(&tx, &input.provider_id, &input.key_id).await?;
    let key_ids = BTreeMap::from([(input.upstream_token_id.clone(), input.key_id.clone())]);
    let model_costs = insert_costs(store, &tx, &input.provider_id, model_ids, key_ids, input.model_costs).await?;
    tx.commit().await?;
    Ok(ProviderQuickImportKeyReplacementRecordOutput {
        api_key,
        model_bindings,
        model_costs,
    })
}

async fn merged_model_ids(
    _store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    mut new_model_ids: BTreeMap<String, String>,
) -> StorageResult<BTreeMap<String, String>> {
    let records = provider_models::Entity::find()
        .filter(provider_models::Column::ProviderId.eq(provider_id))
        .all(tx)
        .await?;
    for record in records {
        new_model_ids.entry(record.global_model_id).or_insert(record.id);
    }
    Ok(new_model_ids)
}

async fn delete_key_model_mappings(tx: &DatabaseTransaction, provider_id: &str, key_id: &str) -> StorageResult<()> {
    provider_quick_import_key_models::Entity::delete_many()
        .filter(provider_quick_import_key_models::Column::ProviderId.eq(provider_id))
        .filter(provider_quick_import_key_models::Column::KeyId.eq(key_id))
        .exec(tx)
        .await?;
    Ok(())
}

async fn insert_replacement_model_mappings(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    input: &ProviderQuickImportKeyReplacementRecordInput,
    key_id: &str,
) -> StorageResult<()> {
    for mapping in input.model_mappings.clone() {
        sync_key_model_active_model(store, &input.provider_id, &input.source_id, key_id, mapping)
            .insert(tx)
            .await?;
    }
    Ok(())
}

async fn update_replacement_api_key(
    tx: &DatabaseTransaction,
    input: &ProviderQuickImportKeyReplacementRecordInput,
) -> StorageResult<types::provider::ProviderApiKey> {
    let record = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(&input.provider_id))
        .filter(provider_api_keys::Column::Id.eq(&input.key_id))
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    let mut active: provider_api_keys::ActiveModel = record.into();
    apply_provider_api_key_patch(&mut active, input.key_patch.clone())?;
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(tx).await?.response()
}

async fn update_replacement_sync_key(tx: &DatabaseTransaction, input: &ProviderQuickImportKeyReplacementRecordInput) -> StorageResult<()> {
    let record = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::ProviderId.eq(&input.provider_id))
        .filter(provider_quick_import_keys::Column::KeyId.eq(&input.key_id))
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    let now = time::OffsetDateTime::now_utc();
    let mut active: provider_quick_import_keys::ActiveModel = record.into();
    active.source_id = Set(input.source_id.clone());
    active.upstream_token_id = Set(input.upstream_token_id.clone());
    active.upstream_token_name = Set(input.upstream_token_name.clone());
    active.upstream_masked_key = Set(input.upstream_masked_key.clone());
    active.upstream_group = Set(input.upstream_group.clone());
    active.upstream_group_ratio = Set(input.upstream_group_ratio);
    active.effective_cost_multiplier = Set(input.effective_cost_multiplier);
    active.sync_statuses = Set(json::encode_required(&vec![ProviderQuickImportSyncStatus::Ok])?);
    active.last_sync_error = Set(None);
    active.last_synced_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(tx).await?;
    Ok(())
}

async fn delete_key_costs(tx: &DatabaseTransaction, provider_id: &str, key_id: &str) -> StorageResult<()> {
    provider_model_costs::Entity::delete_many()
        .filter(provider_model_costs::Column::ProviderId.eq(provider_id))
        .filter(provider_model_costs::Column::KeyId.eq(key_id))
        .exec(tx)
        .await?;
    Ok(())
}

#[derive(Clone)]
struct SyncKeyInput {
    upstream_token_id: String,
    upstream_token_name: String,
    upstream_masked_key: String,
    upstream_group: Option<String>,
    upstream_group_ratio: rust_decimal::Decimal,
    effective_cost_multiplier: rust_decimal::Decimal,
    model_mappings: Vec<ProviderQuickImportKeyModelRecordInput>,
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

async fn insert_sync_metadata(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    source: ProviderQuickImportSourceRecordInput,
    keys: Vec<SyncKeyInput>,
    key_ids: &BTreeMap<String, String>,
) -> StorageResult<()> {
    let source_id = store.next_id();
    sync_source_active_model(source_id.clone(), provider_id, source).insert(tx).await?;
    for key in keys {
        insert_sync_key(store, tx, provider_id, &source_id, key, key_ids).await?;
    }
    Ok(())
}

async fn insert_sync_key(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    source_id: &str,
    input: SyncKeyInput,
    key_ids: &BTreeMap<String, String>,
) -> StorageResult<()> {
    let key_id = mapped_id(key_ids, &input.upstream_token_id, "upstream token")?;
    sync_key_active_model(store, provider_id, source_id, &key_id, &input)?.insert(tx).await?;
    for model in input.model_mappings {
        sync_key_model_active_model(store, provider_id, source_id, &key_id, model).insert(tx).await?;
    }
    Ok(())
}

fn sync_key_inputs(inputs: &[ProviderQuickImportApiKeyRecordInput]) -> Vec<SyncKeyInput> {
    inputs
        .iter()
        .map(|input| SyncKeyInput {
            upstream_token_id: input.upstream_token_id.clone(),
            upstream_token_name: input.upstream_token_name.clone(),
            upstream_masked_key: input.upstream_masked_key.clone(),
            upstream_group: input.upstream_group.clone(),
            upstream_group_ratio: input.upstream_group_ratio,
            effective_cost_multiplier: input.effective_cost_multiplier,
            model_mappings: input.model_mappings.clone(),
        })
        .collect()
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

fn sync_source_active_model(source_id: String, provider_id: &str, input: ProviderQuickImportSourceRecordInput) -> provider_quick_import_sources::ActiveModel {
    let now = time::OffsetDateTime::now_utc();
    provider_quick_import_sources::ActiveModel {
        id: Set(source_id),
        provider_id: Set(provider_id.to_owned()),
        source_kind: Set(input.source_kind),
        base_url: Set(input.base_url),
        encrypted_system_access_token: Set(input.encrypted_system_access_token),
        user_id: Set(input.user_id),
        recharge_multiplier: Set(input.recharge_multiplier),
        auto_sync_enabled: Set(input.sync_config.auto_sync_enabled),
        cost_sync_mode: Set(cost_sync_mode_value(input.sync_config.cost_sync_mode).to_owned()),
        upstream_anomaly_action: Set(anomaly_action_value(input.sync_config.anomaly_actions.token_deleted).to_owned()),
        token_deleted_action: Set(anomaly_action_value(input.sync_config.anomaly_actions.token_deleted).to_owned()),
        token_disabled_action: Set(anomaly_action_value(input.sync_config.anomaly_actions.token_disabled).to_owned()),
        group_removed_action: Set(anomaly_action_value(input.sync_config.anomaly_actions.group_removed).to_owned()),
        group_changed_action: Set(input.sync_config.anomaly_actions.group_changed.as_str().to_owned()),
        key_unavailable_action: Set(anomaly_action_value(input.sync_config.anomaly_actions.key_unavailable).to_owned()),
        model_removed_action: Set(anomaly_action_value(input.sync_config.anomaly_actions.model_removed).to_owned()),
        fetch_failure_action: Set(fetch_failure_action_value(input.sync_config.fetch_failure_action).to_owned()),
        fetch_failure_disable_threshold: Set(input.sync_config.fetch_failure_disable_threshold as i32),
        last_status: Set(None),
        last_error: Set(None),
        last_synced_at: Set(None),
        consecutive_failures: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn sync_key_active_model(
    store: &ProviderStore,
    provider_id: &str,
    source_id: &str,
    key_id: &str,
    input: &SyncKeyInput,
) -> StorageResult<provider_quick_import_keys::ActiveModel> {
    let now = time::OffsetDateTime::now_utc();
    Ok(provider_quick_import_keys::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(provider_id.to_owned()),
        source_id: Set(source_id.to_owned()),
        key_id: Set(key_id.to_owned()),
        upstream_token_id: Set(input.upstream_token_id.clone()),
        upstream_token_name: Set(input.upstream_token_name.clone()),
        upstream_masked_key: Set(input.upstream_masked_key.clone()),
        upstream_group: Set(input.upstream_group.clone()),
        upstream_group_ratio: Set(input.upstream_group_ratio),
        effective_cost_multiplier: Set(input.effective_cost_multiplier),
        sync_statuses: Set(json::encode_required(&vec![ProviderQuickImportSyncStatus::Ok])?),
        last_sync_error: Set(None),
        last_synced_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    })
}

fn sync_key_model_active_model(
    store: &ProviderStore,
    provider_id: &str,
    source_id: &str,
    key_id: &str,
    input: ProviderQuickImportKeyModelRecordInput,
) -> provider_quick_import_key_models::ActiveModel {
    let now = time::OffsetDateTime::now_utc();
    provider_quick_import_key_models::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(provider_id.to_owned()),
        source_id: Set(source_id.to_owned()),
        key_id: Set(key_id.to_owned()),
        upstream_model_id: Set(input.upstream_model_id),
        global_model_id: Set(input.global_model_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
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

fn cost_sync_mode_value(value: types::provider::ProviderQuickImportCostSyncMode) -> &'static str {
    match value {
        types::provider::ProviderQuickImportCostSyncMode::Overwrite => "overwrite",
        types::provider::ProviderQuickImportCostSyncMode::ReportOnly => "report_only",
    }
}

fn anomaly_action_value(value: types::provider::ProviderQuickImportUpstreamAnomalyAction) -> &'static str {
    match value {
        types::provider::ProviderQuickImportUpstreamAnomalyAction::DisableKey => "disable_key",
        types::provider::ProviderQuickImportUpstreamAnomalyAction::ReportOnly => "report_only",
    }
}

fn fetch_failure_action_value(value: types::provider::ProviderQuickImportFetchFailureAction) -> &'static str {
    match value {
        types::provider::ProviderQuickImportFetchFailureAction::ReportOnly => "report_only",
        types::provider::ProviderQuickImportFetchFailureAction::DisableAfterFailures => "disable_after_failures",
    }
}
