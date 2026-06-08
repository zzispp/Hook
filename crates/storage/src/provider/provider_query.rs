use std::collections::HashSet;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use types::provider::{ProviderListRequest, ProviderListResponse};

use crate::{StorageError, StorageResult};

use super::{
    ProviderStore,
    record::{provider_endpoints, provider_models, providers},
    repository_helpers::{ProviderFilterIds, filter_provider_records},
};

pub async fn list_providers(store: &ProviderStore, request: ProviderListRequest) -> StorageResult<ProviderListResponse> {
    let records = provider_records(store).await?;
    let ids = provider_filter_ids(store, &request).await?;
    let records = filter_provider_records(records, &request, ids);
    let total = records.len() as u64;
    let providers = records
        .into_iter()
        .skip(request.skip as usize)
        .take(request.limit as usize)
        .map(Into::into)
        .collect();
    Ok(ProviderListResponse { providers, total })
}

async fn provider_records(store: &ProviderStore) -> StorageResult<Vec<super::ProviderRecord>> {
    providers::Entity::find()
        .order_by_asc(providers::Column::Priority)
        .order_by_asc(providers::Column::Name)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

async fn provider_filter_ids(store: &ProviderStore, request: &ProviderListRequest) -> StorageResult<ProviderFilterIds> {
    Ok(ProviderFilterIds {
        api_format: provider_ids_by_api_format(store, request.api_format.as_deref()).await?,
        model: provider_ids_by_model(store, request.model_id.as_deref()).await?,
    })
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
