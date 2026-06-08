use axum::{
    Json,
    extract::{Path, Query, State},
};
use types::{
    provider::{
        ProviderGroup, ProviderGroupCreate, ProviderGroupListRequest, ProviderGroupListResponse, ProviderGroupUpdate, ProviderKeyGroup, ProviderKeyGroupCreate,
        ProviderKeyGroupListResponse, ProviderKeyGroupUpdate,
    },
    response::ApiResponse,
};

use crate::api::{ProviderApiError, ProviderApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, ProviderApiError>;

pub async fn list_provider_groups(
    State(state): State<ProviderApiState>,
    Query(query): Query<ProviderGroupListRequest>,
) -> ApiResult<ApiJson<ProviderGroupListResponse>> {
    Ok(ok(state.providers.list_provider_groups(query).await?))
}

pub async fn create_provider_group(State(state): State<ProviderApiState>, Json(payload): Json<ProviderGroupCreate>) -> ApiResult<ApiJson<ProviderGroup>> {
    Ok(ok(state.providers.create_provider_group(payload).await?))
}

pub async fn get_provider_group(State(state): State<ProviderApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ProviderGroup>> {
    Ok(ok(state.providers.get_provider_group(&id).await?))
}

pub async fn update_provider_group(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
    Json(payload): Json<ProviderGroupUpdate>,
) -> ApiResult<ApiJson<ProviderGroup>> {
    Ok(ok(state.providers.update_provider_group(&id, payload).await?))
}

pub async fn delete_provider_group(State(state): State<ProviderApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.providers.delete_provider_group(&id).await?;
    Ok(ok(()))
}

pub async fn list_provider_key_groups(
    State(state): State<ProviderApiState>,
    Query(query): Query<ProviderGroupListRequest>,
) -> ApiResult<ApiJson<ProviderKeyGroupListResponse>> {
    Ok(ok(state.providers.list_provider_key_groups(query).await?))
}

pub async fn create_provider_key_group(
    State(state): State<ProviderApiState>,
    Json(payload): Json<ProviderKeyGroupCreate>,
) -> ApiResult<ApiJson<ProviderKeyGroup>> {
    Ok(ok(state.providers.create_provider_key_group(payload).await?))
}

pub async fn get_provider_key_group(State(state): State<ProviderApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ProviderKeyGroup>> {
    Ok(ok(state.providers.get_provider_key_group(&id).await?))
}

pub async fn update_provider_key_group(
    State(state): State<ProviderApiState>,
    Path(id): Path<String>,
    Json(payload): Json<ProviderKeyGroupUpdate>,
) -> ApiResult<ApiJson<ProviderKeyGroup>> {
    Ok(ok(state.providers.update_provider_key_group(&id, payload).await?))
}

pub async fn delete_provider_key_group(State(state): State<ProviderApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.providers.delete_provider_key_group(&id).await?;
    Ok(ok(()))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
