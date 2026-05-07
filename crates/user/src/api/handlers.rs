use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, header::AUTHORIZATION},
};
use constants::auth::{DEFAULT_USER_IS_ACTIVE, DEFAULT_USER_ROLE};
use serde::Serialize;

use crate::api::{ApiState, TokenPair, error::ApiError};
use types::{
    response::ApiResponse,
    user::{ListUsersQuery, NewUser, RefreshTokenPayload, SignInPayload, SignUpPayload, UserId, UserPayload, UserResponse, UsersPageResponse},
};

type ApiResult<T> = Result<T, ApiError>;
type ApiJson<T> = Json<ApiResponse<T>>;

#[derive(Debug, Serialize)]
pub struct AuthSessionResponse {
    user: UserResponse,
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPairResponse {
    access_token: String,
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    user: UserResponse,
}

pub async fn sign_up(State(state): State<ApiState>, Json(payload): Json<SignUpPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    let user = state.users.sign_up(new_sign_up_user(payload)).await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn sign_in(State(state): State<ApiState>, Json(payload): Json<SignInPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    let user = state.users.sign_in(payload.into()).await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn refresh(State(state): State<ApiState>, Json(payload): Json<RefreshTokenPayload>) -> ApiResult<ApiJson<TokenPairResponse>> {
    let (user_id, tokens) = state.tokens.refresh(&payload.refresh_token)?;
    state.users.authenticated_user(user_id).await?;
    Ok(ok(tokens.into()))
}

pub async fn me(State(state): State<ApiState>, headers: HeaderMap) -> ApiResult<ApiJson<MeResponse>> {
    let access_token = bearer_token(&headers)?;
    let user_id = state.tokens.validate_access(access_token)?;
    let user = state.users.authenticated_user(user_id).await?;
    Ok(ok(MeResponse { user: user.into() }))
}

pub async fn create_user(State(state): State<ApiState>, Json(payload): Json<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.create_user(payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn replace_user(State(state): State<ApiState>, Path(id): Path<String>, Json(payload): Json<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.replace_user(UserId(id), payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn delete_user(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
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

fn new_sign_up_user(payload: SignUpPayload) -> NewUser {
    NewUser {
        username: payload.username,
        password: payload.password,
        email: payload.email,
        role: DEFAULT_USER_ROLE.into(),
        is_active: DEFAULT_USER_IS_ACTIVE,
    }
}

impl AuthSessionResponse {
    fn new(user: UserResponse, tokens: TokenPair) -> Self {
        Self {
            user,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }
    }
}

impl From<TokenPair> for TokenPairResponse {
    fn from(value: TokenPair) -> Self {
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
        }
    }
}

fn bearer_token(headers: &HeaderMap) -> ApiResult<&str> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(crate::application::AppError::Unauthorized))?;

    value.strip_prefix("Bearer ").ok_or(ApiError(crate::application::AppError::Unauthorized))
}
