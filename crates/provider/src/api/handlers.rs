use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rbac::api::CurrentUser;
use types::{
    provider::{
        ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCooldown,
        ProviderCooldownListRequest, ProviderCooldownListResponse, ProviderCreate, ProviderEndpoint, ProviderEndpointCreate, ProviderEndpointUpdate,
        ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingBatchUpdate, ProviderModelBindingCreate,
        ProviderModelBindingUpdate, ProviderModelCostBatchUpsert, ProviderModelCostListResponse, ProviderModelTestRequest, ProviderModelTestResponse,
        ProviderUpdate, ProviderUpstreamModelsResponse, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse, UsageRecordListResponse,
    },
    response::ApiResponse,
};

use crate::api::{ProviderApiError, ProviderApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, ProviderApiError>;

pub async fn list_providers(State(state): State<ProviderApiState>, Query(query): Query<ProviderListRequest>) -> ApiResult<ApiJson<ProviderListResponse>> {
    Ok(ok(state.providers.list_providers(query).await?))
}

pub async fn create_provider(State(state): State<ProviderApiState>, Json(payload): Json<ProviderCreate>) -> ApiResult<ApiJson<Provider>> {
    Ok(ok(state.providers.create_provider(payload).await?))
}

pub async fn get_provider(State(state): State<ProviderApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Provider>> {
    Ok(ok(state.providers.get_provider(&id).await?))
}

pub async fn update_provider(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
    Json(payload): Json<ProviderUpdate>,
) -> ApiResult<ApiJson<Provider>> {
    Ok(ok(state.providers.update_provider(&id, payload).await?))
}

pub async fn delete_provider(State(state): State<ProviderApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.providers.delete_provider(&id).await?;
    Ok(ok(()))
}

pub async fn list_provider_cooldowns(
    State(state): State<ProviderApiState>,
    Query(query): Query<ProviderCooldownListRequest>,
) -> ApiResult<ApiJson<ProviderCooldownListResponse>> {
    Ok(ok(state.providers.list_provider_cooldowns(query).await?))
}

pub async fn release_provider_cooldown(State(state): State<ProviderApiState>, Path(provider_id): Path<String>) -> ApiResult<ApiJson<ProviderCooldown>> {
    Ok(ok(state.providers.release_provider_cooldown(&provider_id).await?))
}

pub async fn list_endpoints(State(state): State<ProviderApiState>, Path(provider_id): Path<String>) -> ApiResult<ApiJson<Vec<ProviderEndpoint>>> {
    Ok(ok(state.providers.list_endpoints(&provider_id).await?))
}

pub async fn create_endpoint(
    State(state): State<ProviderApiState>,
    Path(provider_id): Path<String>,
    Json(payload): Json<ProviderEndpointCreate>,
) -> ApiResult<ApiJson<ProviderEndpoint>> {
    Ok(ok(state.providers.create_endpoint(&provider_id, payload).await?))
}

pub async fn update_endpoint(
    State(state): State<ProviderApiState>,
    Path((provider_id, endpoint_id)): Path<(String, String)>,
    Json(payload): Json<ProviderEndpointUpdate>,
) -> ApiResult<ApiJson<ProviderEndpoint>> {
    Ok(ok(state.providers.update_endpoint(&provider_id, &endpoint_id, payload).await?))
}

pub async fn delete_endpoint(State(state): State<ProviderApiState>, Path((provider_id, endpoint_id)): Path<(String, String)>) -> ApiResult<ApiJson<()>> {
    state.providers.delete_endpoint(&provider_id, &endpoint_id).await?;
    Ok(ok(()))
}

pub async fn list_api_keys(State(state): State<ProviderApiState>, Path(provider_id): Path<String>) -> ApiResult<ApiJson<Vec<ProviderApiKey>>> {
    Ok(ok(state.providers.list_api_keys(&provider_id).await?))
}

pub async fn fetch_upstream_models(
    State(state): State<ProviderApiState>,
    Path(provider_id): Path<String>,
) -> ApiResult<ApiJson<ProviderUpstreamModelsResponse>> {
    Ok(ok(state.providers.fetch_upstream_models(&provider_id).await?))
}

pub async fn create_api_key(
    State(state): State<ProviderApiState>,
    Path(provider_id): Path<String>,
    Json(payload): Json<ProviderApiKeyCreate>,
) -> ApiResult<ApiJson<ProviderApiKey>> {
    Ok(ok(state.providers.create_api_key(&provider_id, payload).await?))
}

pub async fn update_api_key(
    State(state): State<ProviderApiState>,
    Path((provider_id, key_id)): Path<(String, String)>,
    Json(payload): Json<ProviderApiKeyUpdate>,
) -> ApiResult<ApiJson<ProviderApiKey>> {
    Ok(ok(state.providers.update_api_key(&provider_id, &key_id, payload).await?))
}

pub async fn delete_api_key(State(state): State<ProviderApiState>, Path((provider_id, key_id)): Path<(String, String)>) -> ApiResult<ApiJson<()>> {
    state.providers.delete_api_key(&provider_id, &key_id).await?;
    Ok(ok(()))
}

pub async fn list_model_bindings(State(state): State<ProviderApiState>, Path(provider_id): Path<String>) -> ApiResult<ApiJson<Vec<ProviderModelBinding>>> {
    Ok(ok(state.providers.list_model_bindings(&provider_id).await?))
}

pub async fn create_model_binding(
    State(state): State<ProviderApiState>,
    Path(provider_id): Path<String>,
    Json(payload): Json<ProviderModelBindingCreate>,
) -> ApiResult<ApiJson<ProviderModelBinding>> {
    Ok(ok(state.providers.create_model_binding(&provider_id, payload).await?))
}

pub async fn batch_update_model_bindings(
    State(state): State<ProviderApiState>,
    Path(provider_id): Path<String>,
    Json(payload): Json<ProviderModelBindingBatchUpdate>,
) -> ApiResult<ApiJson<Vec<ProviderModelBinding>>> {
    Ok(ok(state.providers.batch_update_model_bindings(&provider_id, payload).await?))
}

pub async fn update_model_binding(
    State(state): State<ProviderApiState>,
    Path((provider_id, model_id)): Path<(String, String)>,
    Json(payload): Json<ProviderModelBindingUpdate>,
) -> ApiResult<ApiJson<ProviderModelBinding>> {
    Ok(ok(state.providers.update_model_binding(&provider_id, &model_id, payload).await?))
}

pub async fn delete_model_binding(State(state): State<ProviderApiState>, Path((provider_id, model_id)): Path<(String, String)>) -> ApiResult<ApiJson<()>> {
    state.providers.delete_model_binding(&provider_id, &model_id).await?;
    Ok(ok(()))
}

pub async fn list_model_costs(State(state): State<ProviderApiState>, Path(provider_id): Path<String>) -> ApiResult<ApiJson<ProviderModelCostListResponse>> {
    Ok(ok(state.providers.list_model_costs(&provider_id).await?))
}

pub async fn upsert_model_costs(
    State(state): State<ProviderApiState>,
    Path((provider_id, key_id)): Path<(String, String)>,
    Json(payload): Json<ProviderModelCostBatchUpsert>,
) -> ApiResult<ApiJson<ProviderModelCostListResponse>> {
    Ok(ok(state.providers.upsert_model_costs(&provider_id, &key_id, payload).await?))
}

pub async fn delete_model_cost(
    State(state): State<ProviderApiState>,
    Path((provider_id, key_id, provider_model_id)): Path<(String, String, String)>,
) -> ApiResult<ApiJson<()>> {
    state.providers.delete_model_cost(&provider_id, &key_id, &provider_model_id).await?;
    Ok(ok(()))
}

pub async fn test_model_binding(
    State(state): State<ProviderApiState>,
    Path((provider_id, model_id)): Path<(String, String)>,
    Json(payload): Json<ProviderModelTestRequest>,
) -> ApiResult<ApiJson<ProviderModelTestResponse>> {
    Ok(ok(state.model_tester.test_model_binding(&provider_id, &model_id, payload).await?))
}

pub async fn list_request_records(
    State(state): State<ProviderApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<RequestRecordListRequest>,
) -> ApiResult<ApiJson<RequestRecordListResponse>> {
    let response = state.providers.list_request_records(query).await?;
    Ok(ok(response.with_current_user(&current_user.id, &current_user.username)))
}

pub async fn list_usage_records(
    State(state): State<ProviderApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<RequestRecordListRequest>,
) -> ApiResult<ApiJson<UsageRecordListResponse>> {
    let response = state.providers.list_usage_records(&current_user.id, query).await?;
    Ok(ok(response))
}

pub async fn list_active_request_records(
    State(state): State<ProviderApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<ActiveRequestRecordRequest>,
) -> ApiResult<ApiJson<ActiveRequestRecordResponse>> {
    let response = state.providers.list_active_request_records(payload).await?;
    Ok(ok(response.with_current_user(&current_user.id, &current_user.username)))
}

pub async fn get_request_record(
    State(state): State<ProviderApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(request_id): Path<String>,
) -> ApiResult<ApiJson<RequestRecordDetail>> {
    let response = state.providers.get_request_record(&request_id).await?;
    Ok(ok(response.with_current_user(&current_user.id, &current_user.username)))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

trait RequestRecordCurrentUser {
    fn with_current_user(self, current_user_id: &str, current_username: &str) -> Self;
}

impl RequestRecordCurrentUser for RequestRecordListResponse {
    fn with_current_user(mut self, current_user_id: &str, current_username: &str) -> Self {
        for record in &mut self.records {
            apply_current_user(record, current_user_id, current_username);
        }
        self
    }
}

impl RequestRecordCurrentUser for ActiveRequestRecordResponse {
    fn with_current_user(mut self, current_user_id: &str, current_username: &str) -> Self {
        for record in &mut self.records {
            apply_current_user(record, current_user_id, current_username);
        }
        self
    }
}

impl RequestRecordCurrentUser for RequestRecordDetail {
    fn with_current_user(mut self, current_user_id: &str, current_username: &str) -> Self {
        apply_current_user(&mut self.record, current_user_id, current_username);
        self
    }
}

fn apply_current_user(record: &mut types::provider::RequestRecord, current_user_id: &str, current_username: &str) {
    if record.username.is_none() && record.user_id.as_deref() == Some(current_user_id) {
        record.username = Some(current_username.to_owned());
    }
}
