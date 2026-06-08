use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use storage::{Database, provider::record::provider_api_keys};
use types::group::BillingGroupResponse;

use crate::llm_proxy::LlmProxyError;

pub async fn billing_group_key_scope(database: &Database, group: &BillingGroupResponse) -> Result<Option<Vec<String>>, LlmProxyError> {
    if group.allowed_provider_group_ids.is_empty() && group.allowed_provider_key_group_ids.is_empty() {
        return Ok(None);
    }
    if !group.allowed_provider_key_group_ids.is_empty() {
        return key_group_scope(database, &group.allowed_provider_key_group_ids).await.map(Some);
    }
    provider_group_scope(database, &group.allowed_provider_group_ids).await.map(Some)
}

async fn provider_group_scope(database: &Database, group_ids: &[String]) -> Result<Vec<String>, LlmProxyError> {
    let store = storage::provider::ProviderStore::new(database.clone());
    let provider_ids = store.provider_ids_for_groups(group_ids).await?;
    key_ids_for_providers(database, provider_ids).await
}

async fn key_group_scope(database: &Database, group_ids: &[String]) -> Result<Vec<String>, LlmProxyError> {
    let store = storage::provider::ProviderStore::new(database.clone());
    Ok(store.provider_key_ids_for_key_groups(group_ids).await?)
}

async fn key_ids_for_providers(database: &Database, provider_ids: Vec<String>) -> Result<Vec<String>, LlmProxyError> {
    if provider_ids.is_empty() {
        return Ok(Vec::new());
    }
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.is_in(provider_ids))
        .order_by_asc(provider_api_keys::Column::Id)
        .all(database.connection())
        .await?;
    Ok(records.into_iter().map(|record| record.id).collect())
}
