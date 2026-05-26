use axum::{
    Json,
    extract::{Path, Query, State},
};
use types::{
    pagination::PageRequest,
    response::ApiResponse,
    user::{ListUsersQuery, UserListFilters, UserResponse, UsersPageResponse},
    user_group::{UserGroupCreate, UserGroupListQuery, UserGroupPageResponse, UserGroupResponse, UserGroupUpdate},
};

use crate::api::{ApiState, error::ApiError};

type ApiResult<T> = Result<T, ApiError>;
type ApiJson<T> = Json<ApiResponse<T>>;

pub async fn list_user_groups(State(state): State<ApiState>, Query(query): Query<UserGroupListQuery>) -> ApiResult<ApiJson<UserGroupPageResponse>> {
    Ok(ok(state.user_groups.list_user_groups(query.into()).await?))
}

pub async fn create_user_group(State(state): State<ApiState>, Json(payload): Json<UserGroupCreate>) -> ApiResult<ApiJson<UserGroupResponse>> {
    Ok(ok(state.user_groups.create_user_group(payload).await?))
}

pub async fn get_user_group(State(state): State<ApiState>, Path(code): Path<String>) -> ApiResult<ApiJson<UserGroupResponse>> {
    Ok(ok(state.user_groups.get_user_group(&code).await?))
}

pub async fn update_user_group(
    State(state): State<ApiState>,
    Path(code): Path<String>,
    Json(payload): Json<UserGroupUpdate>,
) -> ApiResult<ApiJson<UserGroupResponse>> {
    Ok(ok(state.user_groups.update_user_group(&code, payload).await?))
}

pub async fn delete_user_group(State(state): State<ApiState>, Path(code): Path<String>) -> ApiResult<ApiJson<()>> {
    state.user_groups.delete_user_group(&code).await?;
    Ok(ok(()))
}

pub async fn list_user_group_members(
    State(state): State<ApiState>,
    Path(code): Path<String>,
    Query(query): Query<ListUsersQuery>,
) -> ApiResult<ApiJson<UsersPageResponse>> {
    let users = state
        .user_groups
        .list_user_group_members(&code, page_request(&query), user_filters(query))
        .await?;
    Ok(ok(UsersPageResponse {
        items: users.items.into_iter().map(UserResponse::from).collect(),
        total: users.total,
        page: users.page,
        page_size: users.page_size,
    }))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

fn page_request(query: &ListUsersQuery) -> PageRequest {
    PageRequest {
        page: query.page,
        page_size: query.page_size,
    }
}

fn user_filters(query: ListUsersQuery) -> UserListFilters {
    UserListFilters {
        search: query.search,
        role: query.role,
        group_code: query.group_code,
        is_active: query.is_active,
    }
}
