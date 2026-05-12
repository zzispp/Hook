use api_token::application::{ApiTokenRepository, hash_token};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Uri, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use types::api_token::ApiToken;

use super::{CurrentApiToken, LlmProxyError, LlmProxyState};

pub async fn token_middleware(State(state): State<LlmProxyState>, mut request: Request, next: Next) -> Result<Response, LlmProxyError> {
    let token = token_value(&request)?;
    let token = authenticate_token(&state, token).await?;
    request.extensions_mut().insert(CurrentApiToken(token));
    Ok(next.run(request).await)
}

async fn authenticate_token(state: &LlmProxyState, value: String) -> Result<ApiToken, LlmProxyError> {
    let token = state.tokens.find_by_hash(&hash_token(&value)).await?.ok_or(LlmProxyError::Unauthorized)?;
    validate_token(token)
}

fn token_value(request: &Request) -> Result<String, LlmProxyError> {
    bearer_token(request.headers())
        .or_else(|| header_token(request.headers(), "x-api-key"))
        .or_else(|| header_token(request.headers(), "x-goog-api-key"))
        .or_else(|| query_key(request.uri()))
        .map(str::to_owned)
        .ok_or(LlmProxyError::Unauthorized)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))?;
    (!value.trim().is_empty()).then_some(value)
}

fn header_token<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    let value = headers.get(key).and_then(|value| value.to_str().ok())?;
    (!value.trim().is_empty()).then_some(value)
}

fn query_key(uri: &Uri) -> Option<&str> {
    uri.query()?.split('&').find_map(query_pair_key)
}

fn query_pair_key(pair: &str) -> Option<&str> {
    let (key, value) = pair.split_once('=')?;
    (key == "key" && !value.trim().is_empty()).then_some(value)
}

fn validate_token(token: ApiToken) -> Result<ApiToken, LlmProxyError> {
    if !token.is_active || token_expired(&token)? {
        return Err(LlmProxyError::Unauthorized);
    }
    Ok(token)
}

fn token_expired(token: &ApiToken) -> Result<bool, LlmProxyError> {
    let Some(expires_at) = token.expires_at.as_deref() else {
        return Ok(false);
    };
    let expires_at =
        OffsetDateTime::parse(expires_at, &Rfc3339).map_err(|error| LlmProxyError::Infrastructure(format!("invalid token expires_at: {error}")))?;
    Ok(expires_at <= OffsetDateTime::now_utc())
}
