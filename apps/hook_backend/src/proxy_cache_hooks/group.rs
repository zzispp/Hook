use async_trait::async_trait;
use group::application::{GroupError, GroupRepository, GroupResult};
use types::group::{BillingGroupCreate, BillingGroupListRequest, BillingGroupListResponse, BillingGroupResponse, BillingGroupUpdate};

use super::cache::ProxyCacheInvalidator;

#[derive(Clone)]
pub struct CachedGroupRepository<R, C> {
    inner: R,
    cache: C,
}

impl<R, C> CachedGroupRepository<R, C> {
    pub const fn new(inner: R, cache: C) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl<R, C> GroupRepository for CachedGroupRepository<R, C>
where
    R: GroupRepository,
    C: ProxyCacheInvalidator,
{
    async fn create_group(&self, input: BillingGroupCreate) -> GroupResult<BillingGroupResponse> {
        let group = self.inner.create_group(input).await?;
        self.refresh_scheduling().await?;
        Ok(group)
    }

    async fn update_group(&self, id: &str, input: BillingGroupUpdate) -> GroupResult<BillingGroupResponse> {
        let group = self.inner.update_group(id, input).await?;
        self.refresh_scheduling().await?;
        Ok(group)
    }

    async fn delete_group(&self, id: &str) -> GroupResult<()> {
        self.inner.delete_group(id).await?;
        self.refresh_scheduling().await
    }

    async fn find_group(&self, id_or_code: &str) -> GroupResult<Option<BillingGroupResponse>> {
        self.inner.find_group(id_or_code).await
    }

    async fn list_groups(&self, request: BillingGroupListRequest) -> GroupResult<BillingGroupListResponse> {
        self.inner.list_groups(request).await
    }

    async fn active_groups(&self) -> GroupResult<Vec<BillingGroupResponse>> {
        self.inner.active_groups().await
    }

    async fn active_groups_for_user_groups(&self, user_group_codes: &[String]) -> GroupResult<Vec<BillingGroupResponse>> {
        self.inner.active_groups_for_user_groups(user_group_codes).await
    }

    async fn group_has_tokens(&self, code: &str) -> GroupResult<bool> {
        self.inner.group_has_tokens(code).await
    }

    async fn user_group_has_billing_groups(&self, user_group_code: &str) -> GroupResult<bool> {
        self.inner.user_group_has_billing_groups(user_group_code).await
    }
}

impl<R, C> CachedGroupRepository<R, C>
where
    C: ProxyCacheInvalidator,
{
    async fn refresh_scheduling(&self) -> GroupResult<()> {
        self.cache.refresh_scheduling().await.map_err(cache_error)
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> GroupError {
    GroupError::Infrastructure(error.to_string())
}
