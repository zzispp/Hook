use types::provider::{ProviderCooldown, ProviderCooldownListRequest, ProviderCooldownListResponse};

use crate::StorageResult;

use super::{ProviderCooldownEventRecordInput, ProviderCooldownRecordInput, repository::ProviderStore};

impl ProviderStore {
    pub async fn upsert_provider_cooldown(&self, input: ProviderCooldownRecordInput) -> StorageResult<ProviderCooldown> {
        super::provider_cooldown_query::upsert_provider_cooldown(self, input).await
    }

    pub async fn create_provider_cooldown_event(&self, input: ProviderCooldownEventRecordInput) -> StorageResult<()> {
        super::provider_cooldown_query::create_provider_cooldown_event(self, input).await
    }

    pub async fn list_active_provider_cooldowns(&self, request: ProviderCooldownListRequest) -> StorageResult<ProviderCooldownListResponse> {
        super::provider_cooldown_query::list_active_provider_cooldowns(self, request).await
    }

    pub async fn release_provider_cooldown(&self, provider_id: &str) -> StorageResult<ProviderCooldown> {
        super::provider_cooldown_query::release_provider_cooldown(self, provider_id).await
    }

    pub async fn active_provider_cooldowns_for_restore(&self) -> StorageResult<Vec<ProviderCooldown>> {
        super::provider_cooldown_query::active_provider_cooldowns_for_restore(self).await
    }
}
