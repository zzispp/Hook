use std::collections::{BTreeMap, BTreeSet};

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set, TransactionTrait};
use types::{model::PatchField, provider::ProviderOrigin};

use crate::{StorageError, StorageResult};

use super::{
    ProviderQuickImportApiKeyRecordInput, ProviderQuickImportBindRecordInput, ProviderQuickImportBindRecordOutput, ProviderQuickImportBoundApiKeyRecordInput,
    ProviderStore,
    quick_import_query::{insert_costs, insert_endpoints, insert_models, insert_sync_metadata, sync_key_inputs},
    record::{
        provider_api_keys, provider_endpoints, provider_key_model_mappings, provider_model_costs, provider_models, provider_quick_import_keys,
        provider_quick_import_sources, providers,
    },
    repository_helpers::apply_provider_api_key_patch,
};

pub async fn bind_quick_import(store: &ProviderStore, input: ProviderQuickImportBindRecordInput) -> StorageResult<ProviderQuickImportBindRecordOutput> {
    let tx = store.connection().begin().await?;
    let provider = provider_record(&tx, &input.provider_id).await?;
    let existing_key_ids = provider_key_ids(&tx, &input.provider_id).await?;
    let reuse_key_ids = reuse_key_ids(&input.api_keys);
    validate_reuse_keys(&existing_key_ids, &reuse_key_ids)?;
    let deleted_key_count = delete_unselected_keys(&tx, &input.provider_id, &reuse_key_ids).await?;
    delete_derived_resources(&tx, &input.provider_id).await?;

    let endpoint_inputs = input.endpoints;
    let model_inputs = input.model_bindings;
    let key_inputs = input.api_keys;
    let sync_key_records = sync_key_record_inputs(&key_inputs);
    let cost_inputs = input.model_costs;
    let endpoints = insert_endpoints(store, &tx, &input.provider_id, endpoint_inputs).await?;
    let (model_bindings, model_ids) = insert_models(store, &tx, &input.provider_id, model_inputs).await?;
    let (api_keys, key_ids) = upsert_keys(store, &tx, &input.provider_id, key_inputs).await?;
    let model_costs = insert_costs(store, &tx, &input.provider_id, &model_ids, key_ids.clone(), cost_inputs).await?;
    insert_sync_metadata(
        store,
        &tx,
        &input.provider_id,
        input.sync_source,
        sync_key_inputs(&sync_key_records),
        &key_ids,
        &model_ids,
    )
    .await?;
    let provider = update_provider_origin(&tx, provider).await?;
    let counts = key_counts(&api_keys, &reuse_key_ids, deleted_key_count);
    tx.commit().await?;

    Ok(ProviderQuickImportBindRecordOutput {
        provider,
        endpoints,
        api_keys,
        model_bindings,
        model_costs,
        created_key_count: counts.created,
        reused_key_count: counts.reused,
        deleted_key_count: counts.deleted,
    })
}

struct KeyCounts {
    created: usize,
    reused: usize,
    deleted: usize,
}

async fn provider_record(tx: &DatabaseTransaction, provider_id: &str) -> StorageResult<providers::Model> {
    providers::Entity::find_by_id(provider_id.to_owned())
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)
}

async fn provider_key_ids(tx: &DatabaseTransaction, provider_id: &str) -> StorageResult<BTreeSet<String>> {
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .all(tx)
        .await?;
    Ok(records.into_iter().map(|record| record.id).collect())
}

fn reuse_key_ids(inputs: &[ProviderQuickImportBoundApiKeyRecordInput]) -> BTreeSet<String> {
    inputs.iter().filter_map(|input| input.local_key_id.clone()).collect()
}

fn sync_key_record_inputs(inputs: &[ProviderQuickImportBoundApiKeyRecordInput]) -> Vec<ProviderQuickImportApiKeyRecordInput> {
    inputs.iter().map(|input| input.input.clone()).collect()
}

fn validate_reuse_keys(existing_key_ids: &BTreeSet<String>, reuse_key_ids: &BTreeSet<String>) -> StorageResult<()> {
    for key_id in reuse_key_ids {
        if !existing_key_ids.contains(key_id) {
            return Err(StorageError::Database(format!("quick import bind key does not belong to provider: {key_id}")));
        }
    }
    Ok(())
}

async fn delete_unselected_keys(tx: &DatabaseTransaction, provider_id: &str, reuse_key_ids: &BTreeSet<String>) -> StorageResult<usize> {
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .all(tx)
        .await?;
    let mut count = 0;
    for record in records {
        if reuse_key_ids.contains(&record.id) {
            continue;
        }
        let active: provider_api_keys::ActiveModel = record.into();
        active.delete(tx).await?;
        count += 1;
    }
    Ok(count)
}

async fn delete_derived_resources(tx: &DatabaseTransaction, provider_id: &str) -> StorageResult<()> {
    provider_key_model_mappings::Entity::delete_many()
        .filter(provider_key_model_mappings::Column::ProviderId.eq(provider_id))
        .exec(tx)
        .await?;
    provider_quick_import_keys::Entity::delete_many()
        .filter(provider_quick_import_keys::Column::ProviderId.eq(provider_id))
        .exec(tx)
        .await?;
    provider_quick_import_sources::Entity::delete_many()
        .filter(provider_quick_import_sources::Column::ProviderId.eq(provider_id))
        .exec(tx)
        .await?;
    provider_model_costs::Entity::delete_many()
        .filter(provider_model_costs::Column::ProviderId.eq(provider_id))
        .exec(tx)
        .await?;
    provider_models::Entity::delete_many()
        .filter(provider_models::Column::ProviderId.eq(provider_id))
        .exec(tx)
        .await?;
    provider_endpoints::Entity::delete_many()
        .filter(provider_endpoints::Column::ProviderId.eq(provider_id))
        .exec(tx)
        .await?;
    Ok(())
}

async fn upsert_keys(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    inputs: Vec<ProviderQuickImportBoundApiKeyRecordInput>,
) -> StorageResult<(Vec<types::provider::ProviderApiKey>, BTreeMap<String, String>)> {
    let mut output = Vec::with_capacity(inputs.len());
    let mut ids = BTreeMap::new();
    for input in inputs {
        let upstream_token_id = input.input.upstream_token_id.clone();
        let record = match input.local_key_id {
            Some(key_id) => update_existing_key(tx, provider_id, &key_id, input.input).await?,
            None => insert_new_key(store, tx, provider_id, input.input).await?,
        };
        ids.insert(upstream_token_id, record.id.clone());
        output.push(record.response()?);
    }
    Ok((output, ids))
}

async fn update_existing_key(
    tx: &DatabaseTransaction,
    provider_id: &str,
    key_id: &str,
    input: super::ProviderQuickImportApiKeyRecordInput,
) -> StorageResult<provider_api_keys::Model> {
    let record = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .filter(provider_api_keys::Column::Id.eq(key_id))
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    let mut active: provider_api_keys::ActiveModel = record.into();
    apply_provider_api_key_patch(&mut active, key_patch(input))?;
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(tx).await.map_err(Into::into)
}

async fn insert_new_key(
    store: &ProviderStore,
    tx: &DatabaseTransaction,
    provider_id: &str,
    input: super::ProviderQuickImportApiKeyRecordInput,
) -> StorageResult<provider_api_keys::Model> {
    let now = time::OffsetDateTime::now_utc();
    provider_api_keys::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(provider_id.to_owned()),
        name: Set(input.name),
        api_formats: Set(crate::json::encode_required(&input.api_formats)?),
        allowed_model_ids: Set(crate::json::encode_required(&input.allowed_model_ids)?),
        encrypted_api_key: Set(input.encrypted_api_key),
        note: Set(input.note),
        internal_priority: Set(input.internal_priority),
        global_priority_by_format: Set(crate::json::encode_required(&input.global_priority_by_format)?),
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
    }
    .insert(tx)
    .await
    .map_err(Into::into)
}

fn key_patch(input: super::ProviderQuickImportApiKeyRecordInput) -> super::ProviderApiKeyRecordPatch {
    super::ProviderApiKeyRecordPatch {
        name: Some(input.name),
        api_formats: Some(input.api_formats),
        allowed_model_ids: Some(input.allowed_model_ids),
        encrypted_api_key: Some(input.encrypted_api_key),
        note: patch_option(input.note),
        internal_priority: Some(input.internal_priority),
        global_priority_by_format: Some(input.global_priority_by_format),
        rpm_limit: patch_option(input.rpm_limit),
        cache_ttl_minutes: Some(input.cache_ttl_minutes),
        max_probe_interval_minutes: Some(input.max_probe_interval_minutes),
        time_range_enabled: Some(input.time_range_enabled),
        time_range_start: patch_option(input.time_range_start),
        time_range_end: patch_option(input.time_range_end),
        is_active: Some(input.is_active),
    }
}

fn patch_option<T>(value: Option<T>) -> PatchField<T> {
    value.map(PatchField::Value).unwrap_or(PatchField::Null)
}

async fn update_provider_origin(tx: &DatabaseTransaction, record: providers::Model) -> StorageResult<types::provider::Provider> {
    let mut active: providers::ActiveModel = record.into();
    active.provider_origin = Set(ProviderOrigin::QuickImport.as_str().to_owned());
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    Ok(active.update(tx).await?.into())
}

fn key_counts(api_keys: &[types::provider::ProviderApiKey], reuse_key_ids: &BTreeSet<String>, deleted: usize) -> KeyCounts {
    let reused = api_keys.iter().filter(|key| reuse_key_ids.contains(&key.id)).count();
    KeyCounts {
        created: api_keys.len().saturating_sub(reused),
        reused,
        deleted,
    }
}
