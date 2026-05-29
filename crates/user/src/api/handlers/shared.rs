use axum::{Json, http::HeaderMap};
use serde::Serialize;
use types::{
    response::ApiResponse,
    user::{AuthSessionData, IdentityProvider, UserResponse},
};

use crate::api::{TokenPair, error::ApiError};
use crate::application::AppError;

pub(super) type ApiResult<T> = Result<T, ApiError>;
pub(super) type ApiJson<T> = Json<ApiResponse<T>>;

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
    pub(super) user: UserResponse,
}

pub(super) fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

impl AuthSessionResponse {
    pub(super) fn new(user: UserResponse, tokens: TokenPair) -> Self {
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

pub(super) fn new_auth_session_data(user: UserResponse, tokens: TokenPair) -> AuthSessionData {
    AuthSessionData {
        user,
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    }
}

pub(super) fn parse_provider(value: &str) -> ApiResult<IdentityProvider> {
    IdentityProvider::try_from(value).map_err(AppError::InvalidInput).map_err(ApiError)
}

pub(super) fn oauth_redirect_uri(headers: &HeaderMap, provider: IdentityProvider) -> ApiResult<String> {
    let host = forwarded_host(headers)?;
    let scheme = forwarded_scheme(headers);
    Ok(format!("{scheme}://{host}/api/auth/oauth/{}/callback", provider.as_str()))
}

pub(super) fn bearer_token(headers: &HeaderMap) -> ApiResult<&str> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(AppError::Unauthorized))?;

    value.strip_prefix("Bearer ").ok_or(ApiError(AppError::Unauthorized))
}

fn forwarded_host(headers: &HeaderMap) -> ApiResult<&str> {
    headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError(AppError::InvalidInput("request host is required".into())))
}

fn forwarded_scheme(headers: &HeaderMap) -> &str {
    headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| *value == "http" || *value == "https")
        .unwrap_or("http")
}
