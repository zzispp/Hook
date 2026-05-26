use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rbac::api::CurrentUser;
use types::{
    group::{BillingGroupCreate, BillingGroupListRequest, BillingGroupListResponse, BillingGroupResponse, BillingGroupUpdate},
    response::ApiResponse,
};

use crate::api::{GroupApiError, GroupApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, GroupApiError>;

pub async fn list_groups(State(state): State<GroupApiState>, Query(query): Query<BillingGroupListRequest>) -> ApiResult<ApiJson<BillingGroupListResponse>> {
    Ok(ok(state.groups.list_groups(query).await?))
}

pub async fn available_groups(
    State(state): State<GroupApiState>,
    Extension(current_user): Extension<CurrentUser>,
) -> ApiResult<ApiJson<Vec<BillingGroupResponse>>> {
    Ok(ok(state.groups.available_groups(&current_user.group_code).await?))
}

pub async fn get_group(State(state): State<GroupApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<BillingGroupResponse>> {
    Ok(ok(state.groups.get_group(&id).await?))
}

pub async fn create_group(State(state): State<GroupApiState>, Json(payload): Json<BillingGroupCreate>) -> ApiResult<ApiJson<BillingGroupResponse>> {
    Ok(ok(state.groups.create_group(payload).await?))
}

pub async fn update_group(
    State(state): State<GroupApiState>,
    Path(id): Path<String>,
    Json(payload): Json<BillingGroupUpdate>,
) -> ApiResult<ApiJson<BillingGroupResponse>> {
    Ok(ok(state.groups.update_group(&id, payload).await?))
}

pub async fn delete_group(State(state): State<GroupApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.groups.delete_group(&id).await?;
    Ok(ok(()))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
