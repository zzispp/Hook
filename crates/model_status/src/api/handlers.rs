use axum::{
    Json,
    extract::{Path, Query, State},
};
use types::{model_status::*, response::ApiResponse};

use crate::api::{ModelStatusApiError, ModelStatusApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, ModelStatusApiError>;

pub async fn list_public(
    State(state): State<ModelStatusApiState>,
    Query(query): Query<ModelStatusListRequest>,
) -> ApiResult<ApiJson<ModelStatusCheckListResponse>> {
    Ok(ok(state.model_status.list_public(query).await?))
}

pub async fn list_admin(
    State(state): State<ModelStatusApiState>,
    Query(query): Query<ModelStatusListRequest>,
) -> ApiResult<ApiJson<ModelStatusCheckListResponse>> {
    Ok(ok(state.model_status.list_admin(query).await?))
}

pub async fn create_check(
    State(state): State<ModelStatusApiState>,
    Json(payload): Json<ModelStatusCheckCreate>,
) -> ApiResult<ApiJson<ModelStatusCheckResponse>> {
    Ok(ok(state.model_status.create_check(payload).await?))
}

pub async fn batch_create_checks(
    State(state): State<ModelStatusApiState>,
    Json(payload): Json<ModelStatusCheckBatchCreateRequest>,
) -> ApiResult<ApiJson<ModelStatusCheckBatchCreateResponse>> {
    Ok(ok(state.model_status.batch_create_checks(payload).await?))
}

pub async fn update_check(
    State(state): State<ModelStatusApiState>,
    Path(id): Path<String>,
    Json(payload): Json<ModelStatusCheckUpdate>,
) -> ApiResult<ApiJson<ModelStatusCheckResponse>> {
    Ok(ok(state.model_status.update_check(&id, payload).await?))
}

pub async fn delete_check(State(state): State<ModelStatusApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.model_status.delete_check(&id).await?;
    Ok(ok(()))
}

pub async fn batch_delete_checks(
    State(state): State<ModelStatusApiState>,
    Json(payload): Json<ModelStatusCheckBatchDeleteRequest>,
) -> ApiResult<ApiJson<ModelStatusCheckBatchDeleteResponse>> {
    Ok(ok(state.model_status.batch_delete_checks(payload.ids).await?))
}

pub async fn batch_update_checks(
    State(state): State<ModelStatusApiState>,
    Json(payload): Json<ModelStatusCheckBatchUpdateRequest>,
) -> ApiResult<ApiJson<ModelStatusCheckBatchUpdateResponse>> {
    Ok(ok(state.model_status.batch_update_checks(payload).await?))
}

pub async fn list_runs(
    State(state): State<ModelStatusApiState>,
    Query(query): Query<ModelStatusRunListRequest>,
) -> ApiResult<ApiJson<ModelStatusRunListResponse>> {
    Ok(ok(state.model_status.list_runs(query).await?))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
