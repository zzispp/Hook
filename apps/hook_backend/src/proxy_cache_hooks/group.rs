use std::sync::Arc;

use async_trait::async_trait;
use group::application::{GroupError, GroupResult, GroupUseCase};
use types::group::{BillingGroupCreate, BillingGroupListRequest, BillingGroupListResponse, BillingGroupResponse, BillingGroupUpdate};

use crate::llm_proxy::LlmProxyCache;

pub struct ProxyCachedGroupUseCase {
    inner: Arc<dyn GroupUseCase>,
    cache: LlmProxyCache,
}

impl ProxyCachedGroupUseCase {
    pub fn new(inner: Arc<dyn GroupUseCase>, cache: LlmProxyCache) -> Self {
        Self { inner, cache }
    }

    async fn refresh_scheduling(&self) -> GroupResult<()> {
        self.cache.refresh_scheduling_snapshot().await.map(|_| ()).map_err(cache_error)
    }
}

#[async_trait]
impl GroupUseCase for ProxyCachedGroupUseCase {
    async fn create_group(&self, input: BillingGroupCreate) -> GroupResult<BillingGroupResponse> {
        let value = self.inner.create_group(input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn update_group(&self, id: &str, input: BillingGroupUpdate) -> GroupResult<BillingGroupResponse> {
        let value = self.inner.update_group(id, input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn delete_group(&self, id: &str) -> GroupResult<()> {
        self.inner.delete_group(id).await?;
        self.refresh_scheduling().await
    }

    async fn get_group(&self, id: &str) -> GroupResult<BillingGroupResponse> {
        self.inner.get_group(id).await
    }

    async fn list_groups(&self, request: BillingGroupListRequest) -> GroupResult<BillingGroupListResponse> {
        self.inner.list_groups(request).await
    }

    async fn available_groups(&self) -> GroupResult<Vec<BillingGroupResponse>> {
        self.inner.available_groups().await
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> GroupError {
    GroupError::Infrastructure(error.to_string())
}
