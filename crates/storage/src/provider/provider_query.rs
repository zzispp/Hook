use std::collections::{HashMap, HashSet};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use types::provider::{ProviderListRequest, ProviderListResponse, ProviderQuickImportAuthMode, ProviderQuickImportSourceSummary};

use crate::{StorageError, StorageResult};

use super::{
    ProviderStore,
    record::{ProviderRecord, provider_endpoints, provider_models, provider_quick_import_sources, providers},
    repository_helpers::{ProviderFilterIds, filter_provider_records},
};

pub async fn list_providers(store: &ProviderStore, request: ProviderListRequest) -> StorageResult<ProviderListResponse> {
    let records = provider_records(store).await?;
    let quick_import_sources = quick_import_source_summaries(store).await?;
    let ids = provider_filter_ids(store, &request).await?;
    let mut records = filter_provider_records(records, &request, ids);
    let total = records.len() as u64;
    sort_provider_records(&mut records);
    let providers = records
        .into_iter()
        .skip(request.skip as usize)
        .take(request.limit as usize)
        .map(|record| provider_response(record, &quick_import_sources))
        .collect();
    Ok(ProviderListResponse { providers, total })
}

fn provider_response(record: ProviderRecord, quick_import_sources: &HashMap<String, ProviderQuickImportSourceSummary>) -> types::provider::Provider {
    let mut provider: types::provider::Provider = record.into();
    provider.quick_import_source = quick_import_sources.get(&provider.id).cloned();
    provider
}

async fn provider_records(store: &ProviderStore) -> StorageResult<Vec<super::ProviderRecord>> {
    providers::Entity::find()
        .order_by_asc(providers::Column::Priority)
        .order_by_asc(providers::Column::Name)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

fn sort_provider_records(records: &mut [ProviderRecord]) {
    records.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.cmp(&right.id))
    });
}

async fn provider_filter_ids(store: &ProviderStore, request: &ProviderListRequest) -> StorageResult<ProviderFilterIds> {
    Ok(ProviderFilterIds {
        api_format: provider_ids_by_api_format(store, request.api_format.as_deref()).await?,
        model: provider_ids_by_model(store, request.model_id.as_deref()).await?,
    })
}

async fn quick_import_source_summaries(store: &ProviderStore) -> StorageResult<HashMap<String, ProviderQuickImportSourceSummary>> {
    let records = provider_quick_import_sources::Entity::find().all(store.connection()).await?;
    Ok(records
        .into_iter()
        .map(|record| {
            let auth_mode = quick_import_auth_mode(&record);
            (
                record.provider_id,
                ProviderQuickImportSourceSummary {
                    source_kind: record.source_kind,
                    auth_mode,
                },
            )
        })
        .collect())
}

fn quick_import_auth_mode(record: &provider_quick_import_sources::Model) -> Option<ProviderQuickImportAuthMode> {
    if !record.email.trim().is_empty() || !record.encrypted_password.trim().is_empty() {
        return Some(ProviderQuickImportAuthMode::Password);
    }
    if !record.encrypted_auth_token.trim().is_empty() || !record.encrypted_refresh_token.trim().is_empty() || record.token_expires_at.is_some() {
        return Some(ProviderQuickImportAuthMode::Token);
    }
    None
}

async fn provider_ids_by_api_format(store: &ProviderStore, api_format: Option<&str>) -> StorageResult<Option<HashSet<String>>> {
    let Some(api_format) = api_format else {
        return Ok(None);
    };
    let records = provider_endpoints::Entity::find()
        .filter(provider_endpoints::Column::ApiFormat.eq(api_format))
        .all(store.connection())
        .await?;
    Ok(Some(records.into_iter().map(|record| record.provider_id).collect()))
}

async fn provider_ids_by_model(store: &ProviderStore, model_id: Option<&str>) -> StorageResult<Option<HashSet<String>>> {
    let Some(model_id) = model_id else {
        return Ok(None);
    };
    let records = provider_models::Entity::find()
        .filter(provider_models::Column::GlobalModelId.eq(model_id))
        .all(store.connection())
        .await?;
    Ok(Some(records.into_iter().map(|record| record.provider_id).collect()))
}
