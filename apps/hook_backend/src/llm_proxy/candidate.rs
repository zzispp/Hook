mod selection;
mod url;

use rust_decimal::Decimal;
use serde_json::Value;
use types::api_token::ApiToken;
use types::model::TieredPricingConfig;

use super::{LlmProxyError, LlmProxyState};

#[derive(Clone, Debug)]
pub struct ProxyCandidate {
    pub trace: CandidateTrace,
    pub requested_model_name: String,
    pub api_key: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub upstream_url: String,
    pub provider_model_name: String,
    pub reasoning_effort: Option<String>,
    pub header_rules: Option<Value>,
    pub body_rules: Option<Value>,
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: TieredPricingConfig,
    pub billing_multiplier: Decimal,
    pub max_retries: i32,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    pub cache_ttl_minutes: i32,
    pub key_rpm_limit: Option<i32>,
    pub route: CandidateRoute,
}

#[derive(Clone, Debug)]
pub struct CandidateRoute {
    pub endpoints: Vec<CandidateEndpointOption>,
    pub keys: Vec<CandidateKeyOption>,
}

#[derive(Clone, Debug)]
pub struct CandidateEndpointOption {
    pub id: String,
    pub provider_api_format: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub upstream_url: String,
    pub header_rules: Option<Value>,
    pub body_rules: Option<Value>,
    pub needs_conversion: bool,
}

#[derive(Clone, Debug)]
pub struct CandidateKeyOption {
    pub id: String,
    pub api_key: String,
    pub cache_ttl_minutes: i32,
    pub rpm_limit: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct CandidateTrace {
    pub token_id: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: String,
    pub provider_id: String,
    pub endpoint_id: String,
    pub key_id: String,
    pub client_api_format: String,
    pub provider_api_format: String,
    pub needs_conversion: bool,
    pub is_stream: bool,
    pub candidate_index: i32,
}

#[derive(Clone, Debug)]
pub struct CandidateSelection {
    pub request_id: String,
    pub candidates: Vec<ProxyCandidate>,
}

#[derive(Clone, Copy)]
pub struct CandidateRequest<'a> {
    pub api_format: &'a str,
    pub model_name: &'a str,
    pub is_stream: bool,
}

pub async fn select_candidates(state: &LlmProxyState, token: &ApiToken, request: CandidateRequest<'_>) -> Result<CandidateSelection, LlmProxyError> {
    selection::select_candidates(state, token, request).await
}

impl ProxyCandidate {
    pub fn for_attempt(&self, retry_index: i32) -> Self {
        let (endpoint_index, key_index) = self.route_indices(retry_index);
        self.with_route_options(&self.route.endpoints[endpoint_index], &self.route.keys[key_index])
    }

    fn route_indices(&self, retry_index: i32) -> (usize, usize) {
        assert!(!self.route.endpoints.is_empty(), "candidate route must contain endpoints");
        assert!(!self.route.keys.is_empty(), "candidate route must contain keys");
        let attempt_index = usize::try_from(retry_index).expect("retry index must be non-negative");
        let key_count = self.route.keys.len();
        let route_count = self
            .route
            .endpoints
            .len()
            .checked_mul(key_count)
            .expect("candidate route option count overflowed");
        let route_index = attempt_index % route_count;
        (route_index / key_count, route_index % key_count)
    }

    fn with_route_options(&self, endpoint: &CandidateEndpointOption, key: &CandidateKeyOption) -> Self {
        let mut candidate = self.clone();
        candidate.trace.endpoint_id = endpoint.id.clone();
        candidate.trace.key_id = key.id.clone();
        candidate.trace.provider_api_format = endpoint.provider_api_format.clone();
        candidate.trace.needs_conversion = endpoint.needs_conversion;
        candidate.api_key = key.api_key.clone();
        candidate.base_url = endpoint.base_url.clone();
        candidate.custom_path = endpoint.custom_path.clone();
        candidate.upstream_url = endpoint.upstream_url.clone();
        candidate.header_rules = endpoint.header_rules.clone();
        candidate.body_rules = endpoint.body_rules.clone();
        candidate.cache_ttl_minutes = key.cache_ttl_minutes;
        candidate
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::model::TieredPricingConfig;

    use super::*;

    #[test]
    fn proxy_candidate_materializes_route_options_by_retry_index() {
        let candidate = route_candidate();

        let first = candidate.for_attempt(0);
        let second = candidate.for_attempt(1);
        let third = candidate.for_attempt(2);
        let cycled = candidate.for_attempt(9);

        assert_eq!(first.trace.endpoint_id, "endpoint-openai");
        assert_eq!(first.trace.key_id, "key-a-1");
        assert_eq!(second.trace.endpoint_id, "endpoint-openai");
        assert_eq!(second.trace.key_id, "key-a-2");
        assert_eq!(third.trace.endpoint_id, "endpoint-gemini");
        assert_eq!(third.trace.key_id, "key-a-1");
        assert_eq!(cycled.trace.endpoint_id, "endpoint-openai");
        assert_eq!(cycled.trace.key_id, "key-a-2");
    }

    fn route_candidate() -> ProxyCandidate {
        ProxyCandidate {
            trace: CandidateTrace {
                token_id: Some("token-a".into()),
                group_code: Some("default".into()),
                global_model_id: "model-a".into(),
                provider_id: "provider-a".into(),
                endpoint_id: "endpoint-openai".into(),
                key_id: "key-a-1".into(),
                client_api_format: "openai_chat".into(),
                provider_api_format: "openai_chat".into(),
                needs_conversion: false,
                is_stream: false,
                candidate_index: 0,
            },
            requested_model_name: "gpt-5.5".into(),
            api_key: "key-1-secret".into(),
            base_url: "https://openai.example.com".into(),
            custom_path: None,
            upstream_url: "https://openai.example.com/v1/chat/completions".into(),
            provider_model_name: "upstream-model".into(),
            reasoning_effort: None,
            header_rules: None,
            body_rules: None,
            price_per_request: None,
            tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
            billing_multiplier: Decimal::ONE,
            max_retries: 3,
            request_timeout_seconds: None,
            stream_first_byte_timeout_seconds: None,
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
            route: CandidateRoute {
                endpoints: vec![
                    endpoint("endpoint-openai", "openai_chat", false),
                    endpoint("endpoint-gemini", "gemini_chat", true),
                ],
                keys: vec![key("key-a-1", "key-1-secret"), key("key-a-2", "key-2-secret")],
            },
        }
    }

    fn endpoint(id: &str, provider_api_format: &str, needs_conversion: bool) -> CandidateEndpointOption {
        CandidateEndpointOption {
            id: id.into(),
            provider_api_format: provider_api_format.into(),
            base_url: format!("https://{id}.example.com"),
            custom_path: None,
            upstream_url: format!("https://{id}.example.com/v1/chat/completions"),
            header_rules: None,
            body_rules: None,
            needs_conversion,
        }
    }

    fn key(id: &str, api_key: &str) -> CandidateKeyOption {
        CandidateKeyOption {
            id: id.into(),
            api_key: api_key.into(),
            cache_ttl_minutes: 5,
            rpm_limit: None,
        }
    }
}
