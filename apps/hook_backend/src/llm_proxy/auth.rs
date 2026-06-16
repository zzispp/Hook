use api_token::application::hash_token;
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
    let token = state.cached_api_token_by_hash(&hash_token(&value)).await?.ok_or(LlmProxyError::Unauthorized)?;
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

pub(super) fn validate_token(token: ApiToken) -> Result<ApiToken, LlmProxyError> {
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

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
    use types::api_token::{ApiToken, ApiTokenType, ModelAccessMode};

    use super::{LlmProxyError, validate_token};

    #[test]
    fn routing_validate_token_accepts_active_unexpired_token() {
        let token = api_token(true, None);

        let validated = validate_token(token).unwrap();

        assert_eq!(validated.id, "token-a");
    }

    #[test]
    fn routing_validate_token_rejects_inactive_token() {
        let error = validate_token(api_token(false, None)).unwrap_err();

        assert!(matches!(error, LlmProxyError::Unauthorized));
    }

    #[test]
    fn routing_validate_token_rejects_expired_token() {
        let expires_at = (OffsetDateTime::now_utc() - Duration::days(1)).format(&Rfc3339).unwrap();
        let error = validate_token(api_token(true, Some(expires_at))).unwrap_err();

        assert!(matches!(error, LlmProxyError::Unauthorized));
    }

    fn api_token(is_active: bool, expires_at: Option<String>) -> ApiToken {
        ApiToken {
            id: "token-a".into(),
            user_id: None,
            token_type: ApiTokenType::Independent,
            name: "Token A".into(),
            token_value: String::new(),
            token_hash: String::new(),
            token_prefix: "sk-test".into(),
            group_code: "default".into(),
            expires_at,
            model_access_mode: ModelAccessMode::All,
            allowed_model_ids: Vec::new(),
            rate_limit_rpm: None,
            quota_limit: None,
            used_quota: Decimal::ZERO,
            request_count: 0,
            is_active,
            last_used_at: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}
