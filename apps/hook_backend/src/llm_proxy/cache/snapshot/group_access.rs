use std::collections::BTreeMap;

use storage::Database;
use types::group::BillingGroupResponse;

use crate::llm_proxy::LlmProxyError;

#[derive(Debug, Default)]
pub struct BillingGroupKeyScope {
    pub allowed_provider_key_ids: Option<Vec<String>>,
    pub provider_priorities: BTreeMap<String, i32>,
    pub provider_key_priorities: BTreeMap<String, i32>,
}

pub async fn billing_group_key_scope(database: &Database, group: &BillingGroupResponse) -> Result<BillingGroupKeyScope, LlmProxyError> {
    if group.allowed_provider_key_group_ids.is_empty() {
        return Ok(BillingGroupKeyScope::default());
    }
    let store = storage::provider::ProviderStore::new(database.clone());
    key_group_scope(&store, &group.allowed_provider_key_group_ids).await
}

async fn key_group_scope(store: &storage::provider::ProviderStore, group_ids: &[String]) -> Result<BillingGroupKeyScope, LlmProxyError> {
    Ok(BillingGroupKeyScope {
        allowed_provider_key_ids: Some(store.provider_key_ids_for_key_groups(group_ids).await?),
        provider_priorities: BTreeMap::new(),
        provider_key_priorities: store.provider_key_priorities_for_key_groups(group_ids).await?,
    })
}
