use axum::{Json, extract::Extension};
use rust_decimal::Decimal;
use types::api_token::{ApiToken, ApiTokenUsageResponse};

use super::{CurrentApiToken, LlmProxyError};

pub async fn usage(Extension(token): Extension<CurrentApiToken>) -> Result<Json<ApiTokenUsageResponse>, LlmProxyError> {
    Ok(Json(token_usage_response(&token.0)))
}

fn token_usage_response(token: &ApiToken) -> ApiTokenUsageResponse {
    ApiTokenUsageResponse {
        used_quota: token.used_quota,
        quota_limit: token.quota_limit,
        remaining_quota: remaining_quota(token.quota_limit, token.used_quota),
    }
}

fn remaining_quota(limit: Option<Decimal>, used: Decimal) -> Option<Decimal> {
    limit.map(|value| if value > used { value - used } else { Decimal::ZERO })
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::api_token::{ApiToken, ApiTokenType, ModelAccessMode};

    use super::token_usage_response;

    #[test]
    fn token_usage_response_clamps_remaining_quota_to_zero() {
        let response = token_usage_response(&token(Some(Decimal::new(100, 0)), Decimal::new(125, 0)));

        assert_eq!(response.used_quota, Decimal::new(125, 0));
        assert_eq!(response.quota_limit, Some(Decimal::new(100, 0)));
        assert_eq!(response.remaining_quota, Some(Decimal::ZERO));
    }

    #[test]
    fn token_usage_response_keeps_unlimited_quota_as_none() {
        let response = token_usage_response(&token(None, Decimal::new(25, 1)));

        assert_eq!(response.quota_limit, None);
        assert_eq!(response.remaining_quota, None);
    }

    fn token(quota_limit: Option<Decimal>, used_quota: Decimal) -> ApiToken {
        ApiToken {
            id: "token-1".into(),
            user_id: None,
            token_type: ApiTokenType::Independent,
            name: "Token".into(),
            token_value: "secret".into(),
            token_hash: "hash".into(),
            token_prefix: "hk-test".into(),
            group_code: "default".into(),
            expires_at: None,
            model_access_mode: ModelAccessMode::All,
            allowed_model_ids: Vec::new(),
            rate_limit_rpm: Some(0),
            quota_limit,
            used_quota,
            request_count: 0,
            is_active: true,
            last_used_at: None,
            created_at: "2026-05-20T00:00:00Z".into(),
            updated_at: "2026-05-20T00:00:00Z".into(),
        }
    }
}
