use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Serialize;

use crate::api::{ApiState, error::ApiError};
use types::{
    response::ApiResponse,
    user::{ListUsersQuery, SignInPayload, UserId, UserPayload, UserResponse, UsersPageResponse},
};

type ApiResult<T> = Result<T, ApiError>;
type ApiJson<T> = Json<ApiResponse<T>>;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health() -> ApiJson<HealthResponse> {
    ok(HealthResponse { status: "ok" })
}

pub async fn sign_up(State(state): State<ApiState>, Json(payload): Json<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.sign_up(payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn sign_in(State(state): State<ApiState>, Json(payload): Json<SignInPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.sign_in(payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn create_user(State(state): State<ApiState>, Json(payload): Json<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.create_user(payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn replace_user(State(state): State<ApiState>, Path(id): Path<u64>, Json(payload): Json<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.replace_user(UserId(id), payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn delete_user(State(state): State<ApiState>, Path(id): Path<u64>) -> ApiResult<ApiJson<()>> {
    state.users.delete_user(UserId(id)).await?;
    Ok(ok(()))
}

pub async fn list_users(State(state): State<ApiState>, Query(query): Query<ListUsersQuery>) -> ApiResult<ApiJson<UsersPageResponse>> {
    let page = state.users.list_users(query.into()).await?;
    Ok(ok(page.into()))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
