use std::collections::HashSet;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderEndpointRecordInput, ProviderEndpointRecordPatch, ProviderStore,
    record::{provider_api_keys, provider_endpoints},
    repository_helpers::{apply_endpoint_patch, endpoint_belongs_to_provider},
};

pub async fn create_endpoint(store: &ProviderStore, input: ProviderEndpointRecordInput) -> StorageResult<types::provider::ProviderEndpoint> {
    let now = time::OffsetDateTime::now_utc();
    let record = provider_endpoints::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(input.provider_id),
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
    }
    .insert(store.connection())
    .await?;
    record.response()
}

pub async fn endpoints_for_provider(store: &ProviderStore, provider_id: &str) -> StorageResult<Vec<types::provider::ProviderEndpoint>> {
    let records = provider_endpoints::Entity::find()
        .filter(provider_endpoints::Column::ProviderId.eq(provider_id))
        .order_by_asc(provider_endpoints::Column::ApiFormat)
        .all(store.connection())
        .await?;
    records.into_iter().map(|record| record.response()).collect()
}

pub async fn update_endpoint(
    store: &ProviderStore,
    provider_id: &str,
    endpoint_id: &str,
    input: ProviderEndpointRecordPatch,
) -> StorageResult<types::provider::ProviderEndpoint> {
    let record = endpoint_record(store, provider_id, endpoint_id).await?;
    let mut active: provider_endpoints::ActiveModel = record.into();
    apply_endpoint_patch(&mut active, input)?;
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    let record = active.update(store.connection()).await?;
    prune_api_key_formats_to_provider_endpoints(store, provider_id).await?;
    record.response()
}

pub async fn delete_endpoint(store: &ProviderStore, provider_id: &str, endpoint_id: &str) -> StorageResult<()> {
    let record = endpoint_record(store, provider_id, endpoint_id).await?;
    let active: provider_endpoints::ActiveModel = record.into();
    active.delete(store.connection()).await?;
    prune_api_key_formats_to_provider_endpoints(store, provider_id).await
}

async fn endpoint_record(store: &ProviderStore, provider_id: &str, endpoint_id: &str) -> StorageResult<super::ProviderEndpointRecord> {
    let record = provider_endpoints::Entity::find_by_id(endpoint_id.to_owned())
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    if !endpoint_belongs_to_provider(&record, provider_id) {
        return Err(StorageError::NotFound);
    }
    Ok(record)
}

async fn prune_api_key_formats_to_provider_endpoints(store: &ProviderStore, provider_id: &str) -> StorageResult<()> {
    let endpoint_formats = endpoint_formats_for_provider(store, provider_id).await?;
    let keys = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .all(store.connection())
        .await?;

    for key in keys {
        let api_formats = json::decode_required(key.api_formats.clone())?;
        let pruned = retain_bound_formats(api_formats, &endpoint_formats);
        if !pruned.changed {
            continue;
        }
        let mut active: provider_api_keys::ActiveModel = key.into();
        active.api_formats = Set(json::encode_required(&pruned.values)?);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(store.connection()).await?;
    }
    Ok(())
}

async fn endpoint_formats_for_provider(store: &ProviderStore, provider_id: &str) -> StorageResult<HashSet<String>> {
    let endpoints = provider_endpoints::Entity::find()
        .filter(provider_endpoints::Column::ProviderId.eq(provider_id))
        .all(store.connection())
        .await?;
    Ok(endpoints.into_iter().map(|endpoint| endpoint.api_format).collect())
}

struct PrunedKeyFormats {
    values: Vec<String>,
    changed: bool,
}

fn retain_bound_formats(api_formats: Vec<String>, endpoint_formats: &HashSet<String>) -> PrunedKeyFormats {
    let original_len = api_formats.len();
    let values = api_formats
        .into_iter()
        .filter(|api_format| endpoint_formats.contains(api_format))
        .collect::<Vec<_>>();
    PrunedKeyFormats {
        changed: values.len() != original_len,
        values,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retain_bound_formats_removes_formats_without_provider_endpoint() {
        let endpoint_formats = HashSet::from(["openai:chat".to_owned(), "gemini:cli".to_owned()]);

        let output = retain_bound_formats(vec!["openai:chat".to_owned(), "claude:chat".to_owned()], &endpoint_formats);

        assert!(output.changed);
        assert_eq!(output.values, vec!["openai:chat"]);
    }

    #[test]
    fn retain_bound_formats_preserves_bound_formats() {
        let endpoint_formats = HashSet::from(["openai:chat".to_owned()]);

        let output = retain_bound_formats(vec!["openai:chat".to_owned()], &endpoint_formats);

        assert!(!output.changed);
        assert_eq!(output.values, vec!["openai:chat"]);
    }
}
