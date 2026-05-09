use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rbac::api::CurrentUser;
use types::{
    api_token::{
        AdminApiTokenCreate, ApiTokenCreate, ApiTokenCreateResponse, ApiTokenListRequest, ApiTokenListResponse, ApiTokenResponse, ApiTokenSecretResponse,
        ApiTokenUpdate,
    },
    response::ApiResponse,
};

use crate::api::{ApiTokenApiError, ApiTokenApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, ApiTokenApiError>;

pub async fn list_tokens(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ApiTokenListRequest>,
) -> ApiResult<ApiJson<ApiTokenListResponse>> {
    state.tokens.cleanup_expired_tokens().await?;
    Ok(ok(state.tokens.list_tokens(&current_user.id, query).await?))
}

pub async fn get_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<ApiTokenResponse>> {
    Ok(ok(state.tokens.get_token(&current_user.id, &id).await?))
}

pub async fn token_secret(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<ApiTokenSecretResponse>> {
    Ok(ok(state.tokens.token_secret(&current_user.id, &id).await?))
}

pub async fn create_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<ApiTokenCreate>,
) -> ApiResult<ApiJson<ApiTokenCreateResponse>> {
    Ok(ok(state.tokens.create_token(&current_user.id, payload).await?))
}

pub async fn update_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(payload): Json<ApiTokenUpdate>,
) -> ApiResult<ApiJson<ApiTokenResponse>> {
    Ok(ok(state.tokens.update_token(&current_user.id, &id, payload).await?))
}

pub async fn delete_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<()>> {
    state.tokens.delete_token(&current_user.id, &id).await?;
    Ok(ok(()))
}

pub async fn list_admin_tokens(State(state): State<ApiTokenApiState>, Query(query): Query<ApiTokenListRequest>) -> ApiResult<ApiJson<ApiTokenListResponse>> {
    state.tokens.cleanup_expired_tokens().await?;
    Ok(ok(state.tokens.list_admin_tokens(query).await?))
}

pub async fn get_admin_token(State(state): State<ApiTokenApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ApiTokenResponse>> {
    Ok(ok(state.tokens.get_admin_token(&id).await?))
}

pub async fn admin_token_secret(State(state): State<ApiTokenApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ApiTokenSecretResponse>> {
    Ok(ok(state.tokens.admin_token_secret(&id).await?))
}

pub async fn create_admin_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AdminApiTokenCreate>,
) -> ApiResult<ApiJson<ApiTokenCreateResponse>> {
    Ok(ok(state.tokens.create_admin_token(&current_user.id, payload).await?))
}

pub async fn update_admin_token(
    State(state): State<ApiTokenApiState>,
    Path(id): Path<String>,
    Json(payload): Json<ApiTokenUpdate>,
) -> ApiResult<ApiJson<ApiTokenResponse>> {
    Ok(ok(state.tokens.update_admin_token(&id, payload).await?))
}

pub async fn delete_admin_token(State(state): State<ApiTokenApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.tokens.delete_admin_token(&id).await?;
    Ok(ok(()))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
