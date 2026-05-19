use async_trait::async_trait;

use crate::llm_proxy::{LlmProxyCache, LlmProxyError};

#[async_trait]
pub(crate) trait ProxyCacheInvalidator: Send + Sync + 'static {
    async fn refresh_scheduling(&self) -> Result<(), LlmProxyError>;
    async fn bump_auth(&self) -> Result<(), LlmProxyError>;
    async fn clear_provider_cooldown(&self, provider_id: &str) -> Result<(), LlmProxyError>;
}

#[async_trait]
impl ProxyCacheInvalidator for LlmProxyCache {
    async fn refresh_scheduling(&self) -> Result<(), LlmProxyError> {
        LlmProxyCache::refresh_scheduling_snapshot(self).await.map(|_| ())
    }

    async fn bump_auth(&self) -> Result<(), LlmProxyError> {
        LlmProxyCache::bump_auth_version(self).await
    }

    async fn clear_provider_cooldown(&self, provider_id: &str) -> Result<(), LlmProxyError> {
        LlmProxyCache::clear_provider_cooldown(self, provider_id).await
    }
}

pub(crate) fn combine_cache_results(first: Result<(), LlmProxyError>, second: Result<(), LlmProxyError>) -> Result<(), LlmProxyError> {
    match (first, second) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(first), Err(second)) => Err(LlmProxyError::Infrastructure(format!("{first}; additionally failed cache operation: {second}"))),
    }
}
