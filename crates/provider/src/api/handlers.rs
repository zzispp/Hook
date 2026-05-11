use axum::{
    Json,
    extract::{Path, Query, State},
};
use types::{
    provider::{
        ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderApiKey, ProviderApiKeyCreate, ProviderCreate, ProviderEndpoint,
        ProviderEndpointCreate, ProviderEndpointUpdate, ProviderListRequest, ProviderListResponse, ProviderModelBinding, ProviderModelBindingCreate,
        ProviderUpdate, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse,
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

pub async fn create_api_key(
    State(state): State<ProviderApiState>,
    Path(provider_id): Path<String>,
    Json(payload): Json<ProviderApiKeyCreate>,
) -> ApiResult<ApiJson<ProviderApiKey>> {
    Ok(ok(state.providers.create_api_key(&provider_id, payload).await?))
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

pub async fn list_request_records(
    State(state): State<ProviderApiState>,
    Query(query): Query<RequestRecordListRequest>,
) -> ApiResult<ApiJson<RequestRecordListResponse>> {
    Ok(ok(state.providers.list_request_records(query).await?))
}

pub async fn list_active_request_records(
    State(state): State<ProviderApiState>,
    Json(payload): Json<ActiveRequestRecordRequest>,
) -> ApiResult<ApiJson<ActiveRequestRecordResponse>> {
    Ok(ok(state.providers.list_active_request_records(payload).await?))
}

pub async fn get_request_record(State(state): State<ProviderApiState>, Path(request_id): Path<String>) -> ApiResult<ApiJson<RequestRecordDetail>> {
    Ok(ok(state.providers.get_request_record(&request_id).await?))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
