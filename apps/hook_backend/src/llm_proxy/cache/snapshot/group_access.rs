use std::collections::BTreeMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use storage::{Database, provider::record::provider_api_keys};
use types::group::BillingGroupResponse;

use crate::llm_proxy::LlmProxyError;

#[derive(Debug, Default)]
pub struct BillingGroupKeyScope {
    pub allowed_provider_key_ids: Option<Vec<String>>,
    pub provider_priorities: BTreeMap<String, i32>,
    pub provider_key_priorities: BTreeMap<String, i32>,
}

pub async fn billing_group_key_scope(database: &Database, group: &BillingGroupResponse) -> Result<BillingGroupKeyScope, LlmProxyError> {
    if group.allowed_provider_group_ids.is_empty() && group.allowed_provider_key_group_ids.is_empty() {
        return Ok(BillingGroupKeyScope::default());
    }
    let store = storage::provider::ProviderStore::new(database.clone());
    if !group.allowed_provider_key_group_ids.is_empty() {
        return key_group_scope(&store, &group.allowed_provider_key_group_ids).await;
    }
    provider_group_scope(database, &store, &group.allowed_provider_group_ids).await
}

async fn provider_group_scope(
    database: &Database,
    store: &storage::provider::ProviderStore,
    group_ids: &[String],
) -> Result<BillingGroupKeyScope, LlmProxyError> {
    let provider_ids = store.provider_ids_for_groups(group_ids).await?;
    Ok(BillingGroupKeyScope {
        allowed_provider_key_ids: Some(key_ids_for_providers(database, provider_ids).await?),
        provider_priorities: store.provider_priorities_for_groups(group_ids).await?,
        provider_key_priorities: BTreeMap::new(),
    })
}

async fn key_group_scope(store: &storage::provider::ProviderStore, group_ids: &[String]) -> Result<BillingGroupKeyScope, LlmProxyError> {
    Ok(BillingGroupKeyScope {
        allowed_provider_key_ids: Some(store.provider_key_ids_for_key_groups(group_ids).await?),
        provider_priorities: BTreeMap::new(),
        provider_key_priorities: store.provider_key_priorities_for_key_groups(group_ids).await?,
    })
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
