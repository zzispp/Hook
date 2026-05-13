use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, header::AUTHORIZATION},
};
use constants::auth::{DEFAULT_USER_IS_ACTIVE, DEFAULT_USER_ROLE};
use serde::Serialize;

use crate::api::{ApiState, TokenPair, error::ApiError};
use crate::application::AppError;
use types::{
    pagination::PageRequest,
    response::ApiResponse,
    user::{
        ListUsersQuery, NewUser, RefreshTokenPayload, SignInPayload, SignUpPayload, USER_QUOTA_MODE_WALLET, User, UserId, UserListFilters, UserPayload,
        UserResponse, UsersPageResponse,
    },
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
    verify_registration_captcha(&state, payload.captcha_token.as_deref()).await?;
    let user = state.users.sign_up(new_sign_up_user(payload)).await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn sign_in(State(state): State<ApiState>, Json(payload): Json<SignInPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    verify_login_captcha(&state, payload.captcha_token.as_deref()).await?;
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
    let page = PageRequest {
        page: query.page,
        page_size: query.page_size,
    };
    let filters = UserListFilters {
        search: query.search,
        role: query.role,
        is_active: query.is_active,
    };
    let page = state.users.list_users(page, filters).await?;
    let wallets = state.users.wallet_summaries(&user_ids(&page.items)).await?;
    let response = UsersPageResponse {
        items: page
            .items
            .into_iter()
            .map(|user| {
                let wallet = wallets.get(&user.id.0).cloned();
                UserResponse::from(user).with_wallet(wallet)
            })
            .collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    };
    Ok(ok(response))
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
        allowed_model_ids: Vec::new(),
        allowed_provider_ids: Vec::new(),
        rate_limit_rpm: None,
        quota_mode: USER_QUOTA_MODE_WALLET.into(),
    }
}

fn user_ids(users: &[User]) -> Vec<String> {
    users.iter().filter(|user| !user.system).map(|user| user.id.0.clone()).collect()
}

async fn verify_login_captcha(state: &ApiState, token: Option<&str>) -> ApiResult<()> {
    state.captcha.verify_login(token).await.map_err(captcha_error)
}

async fn verify_registration_captcha(state: &ApiState, token: Option<&str>) -> ApiResult<()> {
    state.captcha.verify_registration(token).await.map_err(captcha_error)
}

fn captcha_error(error: captcha::application::CaptchaError) -> ApiError {
    match error {
        captcha::application::CaptchaError::InvalidInput(message) => ApiError(AppError::InvalidInput(message)),
        captcha::application::CaptchaError::Infrastructure(message) => ApiError(AppError::Infrastructure(message)),
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
