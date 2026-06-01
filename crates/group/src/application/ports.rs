use async_trait::async_trait;
use types::group::{BillingGroupCreate, BillingGroupListRequest, BillingGroupListResponse, BillingGroupResponse, BillingGroupUpdate};

use super::GroupResult;

#[async_trait]
pub trait GroupRepository: Send + Sync + 'static {
    async fn create_group(&self, input: BillingGroupCreate) -> GroupResult<BillingGroupResponse>;
    async fn update_group(&self, id: &str, input: BillingGroupUpdate) -> GroupResult<BillingGroupResponse>;
    async fn delete_group(&self, id: &str) -> GroupResult<()>;
    async fn find_group(&self, id_or_code: &str) -> GroupResult<Option<BillingGroupResponse>>;
    async fn list_groups(&self, request: BillingGroupListRequest) -> GroupResult<BillingGroupListResponse>;
    async fn active_groups(&self) -> GroupResult<Vec<BillingGroupResponse>>;
    async fn active_groups_for_user_groups(&self, user_group_codes: &[String]) -> GroupResult<Vec<BillingGroupResponse>>;
    async fn group_has_tokens(&self, code: &str) -> GroupResult<bool>;
    async fn user_group_has_billing_groups(&self, user_group_code: &str) -> GroupResult<bool>;
}

#[async_trait]
pub trait GroupModelCatalog: Send + Sync + 'static {
    async fn model_exists(&self, id: &str) -> GroupResult<bool>;
}

#[async_trait]
pub trait GroupProviderCatalog: Send + Sync + 'static {
    async fn provider_exists(&self, id: &str) -> GroupResult<bool>;
    async fn provider_key_provider_id(&self, id: &str) -> GroupResult<Option<String>>;
}

#[async_trait]
pub trait GroupUserGroupCatalog: Send + Sync + 'static {
    async fn active_user_group_exists(&self, code: &str) -> GroupResult<bool>;
}

#[async_trait]
pub trait GroupUseCase: Send + Sync + 'static {
    async fn create_group(&self, input: BillingGroupCreate) -> GroupResult<BillingGroupResponse>;
    async fn update_group(&self, id: &str, input: BillingGroupUpdate) -> GroupResult<BillingGroupResponse>;
    async fn delete_group(&self, id: &str) -> GroupResult<()>;
    async fn get_group(&self, id: &str) -> GroupResult<BillingGroupResponse>;
    async fn list_groups(&self, request: BillingGroupListRequest) -> GroupResult<BillingGroupListResponse>;
    async fn available_groups(&self, user_group_codes: &[String]) -> GroupResult<Vec<BillingGroupResponse>>;
}
