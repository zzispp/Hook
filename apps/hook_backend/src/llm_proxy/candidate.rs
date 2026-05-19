mod selection;
pub(super) mod url;

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
    pub stream_idle_timeout_seconds: Option<f64>,
    pub cache_ttl_minutes: i32,
    pub key_rpm_limit: Option<i32>,
    pub is_cached: bool,
    pub route: CandidateRoute,
}

#[derive(Clone, Debug)]
pub struct CandidateRoute {
    pub options: Vec<CandidateRouteOption>,
}

#[derive(Clone, Debug)]
pub struct CandidateRouteOption {
    pub endpoint: CandidateEndpointOption,
    pub key: CandidateKeyOption,
}

#[derive(Clone, Debug)]
pub struct CandidateEndpointOption {
    pub id: String,
    pub name: String,
    pub provider_api_format: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub upstream_url: String,
    pub max_retries: Option<i32>,
    pub header_rules: Option<Value>,
    pub body_rules: Option<Value>,
    pub needs_conversion: bool,
}

#[derive(Clone, Debug)]
pub struct CandidateKeyOption {
    pub id: String,
    pub name: String,
    pub key_preview: String,
    pub api_key: String,
    pub cache_ttl_minutes: i32,
    pub rpm_limit: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct CandidateTrace {
    pub token_id: Option<String>,
    pub user_id_snapshot: Option<String>,
    pub username_snapshot: Option<String>,
    pub token_name_snapshot: Option<String>,
    pub token_prefix_snapshot: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: String,
    pub provider_model_id: String,
    pub model_name_snapshot: String,
    pub provider_id: String,
    pub provider_name_snapshot: String,
    pub endpoint_id: String,
    pub endpoint_name_snapshot: String,
    pub key_id: String,
    pub key_name_snapshot: String,
    pub key_preview_snapshot: String,
    pub client_api_format: String,
    pub provider_api_format: String,
    pub needs_conversion: bool,
    pub is_stream: bool,
    pub is_cached: bool,
    pub candidate_index: i32,
}

#[derive(Clone, Debug)]
pub struct CandidateSelection {
    pub request_id: String,
    pub cache_affinity_ttl_minutes: i64,
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
        let option_index = self.route_index(retry_index);
        self.with_route_option(&self.route.options[option_index])
    }

    fn route_index(&self, retry_index: i32) -> usize {
        assert!(!self.route.options.is_empty(), "candidate route must contain options");
        let attempt_index = usize::try_from(retry_index).expect("retry index must be non-negative");
        attempt_index % self.route.options.len()
    }

    fn with_route_option(&self, option: &CandidateRouteOption) -> Self {
        let mut candidate = self.clone();
        let endpoint = &option.endpoint;
        let key = &option.key;
        candidate.trace.endpoint_id = endpoint.id.clone();
        candidate.trace.endpoint_name_snapshot = endpoint.name.clone();
        candidate.trace.key_id = key.id.clone();
        candidate.trace.key_name_snapshot = key.name.clone();
        candidate.trace.key_preview_snapshot = key.key_preview.clone();
        candidate.trace.provider_api_format = endpoint.provider_api_format.clone();
        candidate.trace.needs_conversion = endpoint.needs_conversion;
        candidate.api_key = key.api_key.clone();
        candidate.base_url = endpoint.base_url.clone();
        candidate.custom_path = endpoint.custom_path.clone();
        candidate.upstream_url = endpoint.upstream_url.clone();
        candidate.header_rules = endpoint.header_rules.clone();
        candidate.body_rules = endpoint.body_rules.clone();
        candidate.cache_ttl_minutes = key.cache_ttl_minutes;
        candidate.key_rpm_limit = key.rpm_limit;
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
                user_id_snapshot: Some("user-a".into()),
                username_snapshot: Some("alice".into()),
                token_name_snapshot: Some("token-a-name".into()),
                token_prefix_snapshot: Some("sk-a".into()),
                group_code: Some("default".into()),
                global_model_id: "model-a".into(),
                provider_model_id: "provider-model-a".into(),
                model_name_snapshot: "model-a".into(),
                provider_id: "provider-a".into(),
                provider_name_snapshot: "provider-a-name".into(),
                endpoint_id: "endpoint-openai".into(),
                endpoint_name_snapshot: "openai_chat".into(),
                key_id: "key-a-1".into(),
                key_name_snapshot: "key-a-1-name".into(),
                key_preview_snapshot: "***cret".into(),
                client_api_format: "openai_chat".into(),
                provider_api_format: "openai_chat".into(),
                needs_conversion: false,
                is_stream: false,
                is_cached: false,
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
            stream_idle_timeout_seconds: None,
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
            is_cached: false,
            route: CandidateRoute {
                options: vec![
                    route_option(endpoint("endpoint-openai", "openai_chat", false), key("key-a-1", "key-1-secret")),
                    route_option(endpoint("endpoint-openai", "openai_chat", false), key("key-a-2", "key-2-secret")),
                    route_option(endpoint("endpoint-gemini", "gemini_chat", true), key("key-a-1", "key-1-secret")),
                    route_option(endpoint("endpoint-gemini", "gemini_chat", true), key("key-a-2", "key-2-secret")),
                ],
            },
        }
    }

    fn route_option(endpoint: CandidateEndpointOption, key: CandidateKeyOption) -> CandidateRouteOption {
        CandidateRouteOption { endpoint, key }
    }

    fn endpoint(id: &str, provider_api_format: &str, needs_conversion: bool) -> CandidateEndpointOption {
        CandidateEndpointOption {
            id: id.into(),
            name: provider_api_format.into(),
            provider_api_format: provider_api_format.into(),
            base_url: format!("https://{id}.example.com"),
            custom_path: None,
            upstream_url: format!("https://{id}.example.com/v1/chat/completions"),
            max_retries: None,
            header_rules: None,
            body_rules: None,
            needs_conversion,
        }
    }

    fn key(id: &str, api_key: &str) -> CandidateKeyOption {
        CandidateKeyOption {
            id: id.into(),
            name: format!("{id}-name"),
            key_preview: "***cret".into(),
            api_key: api_key.into(),
            cache_ttl_minutes: 5,
            rpm_limit: None,
        }
    }
}
