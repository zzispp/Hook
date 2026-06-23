mod selection;
pub(super) mod url;

use rust_decimal::Decimal;
use serde_json::Value;
use types::api_token::ApiToken;
use types::model::TieredPricingConfig;
use types::provider::{RouteIdentity, RouteScoreExplanation, RoutingProfileId, RoutingRankingResponse, RoutingRankingsRequest, RoutingRequestFeatures};

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
    pub header_rules: Option<serde_json::Value>,
    pub body_rules: Option<serde_json::Value>,
    pub format_acceptance_config: Option<serde_json::Value>,
    pub key_supports_image_generation: bool,
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
    pub format_acceptance_config: Option<Value>,
    pub needs_conversion: bool,
}

#[derive(Clone, Debug)]
pub struct CandidateKeyOption {
    pub id: String,
    pub name: String,
    pub key_preview: String,
    pub api_key: String,
    pub supports_image_generation: bool,
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
    pub routing_profile_id: RoutingProfileId,
    pub routing_profile_ema_alpha: f64,
    pub routing_context_key: String,
    pub route_config_fingerprint: String,
    pub price_config_fingerprint: String,
    pub candidate_index: i32,
}

#[derive(Clone, Debug)]
pub struct CandidateSelection {
    pub request_id: String,
    pub cache_affinity_ttl_minutes: i64,
    pub routing_profile_id: Option<RoutingProfileId>,
    pub routing_profile_version: Option<String>,
    pub routing_explanations: Vec<RouteScoreExplanation>,
    pub candidates: Vec<ProxyCandidate>,
}

#[derive(Clone)]
pub struct CandidateRequest<'a> {
    pub api_format: &'a str,
    pub routing_api_format: &'a str,
    pub model_name: &'a str,
    pub is_stream: bool,
    pub has_openai_responses_custom_tool_items: bool,
    pub has_openai_responses_tool_outputs_without_previous_response_id: bool,
    pub features: RoutingRequestFeatures,
}

pub async fn select_candidates(state: &LlmProxyState, token: &ApiToken, request: CandidateRequest<'_>) -> Result<CandidateSelection, LlmProxyError> {
    selection::select_candidates(state, token, request).await
}

pub(crate) async fn routing_rankings(state: &LlmProxyState, request: RoutingRankingsRequest) -> Result<RoutingRankingResponse, LlmProxyError> {
    selection::routing_rankings(state, request).await
}

impl ProxyCandidate {
    pub fn for_attempt(&self, retry_index: i32) -> Self {
        let option_index = self.route_index(retry_index);
        self.with_route_option(&self.route.options[option_index])
    }

    pub fn route_retry_floor(&self) -> i32 {
        let option_count = i32::try_from(self.route.options.len()).expect("candidate route option count must fit retry index range");
        option_count.saturating_sub(1)
    }

    pub fn max_attempt_index(&self) -> i32 {
        let route_retry_floor = self.route_retry_floor();
        if !self.is_cached {
            return route_retry_floor;
        }
        self.max_retries.max(route_retry_floor)
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
        candidate.format_acceptance_config = endpoint.format_acceptance_config.clone();
        candidate.key_supports_image_generation = key.supports_image_generation;
        candidate.cache_ttl_minutes = key.cache_ttl_minutes;
        candidate.key_rpm_limit = key.rpm_limit;
        candidate
    }
}

impl CandidateTrace {
    pub fn route_identity(&self) -> RouteIdentity {
        RouteIdentity {
            provider_id: self.provider_id.clone(),
            key_id: self.key_id.clone(),
            endpoint_id: self.endpoint_id.clone(),
            global_model_id: self.global_model_id.clone(),
            client_api_format: self.client_api_format.clone(),
            provider_api_format: self.provider_api_format.clone(),
            is_stream: self.is_stream,
        }
    }
}

#[cfg(test)]
#[path = "candidate_tests.rs"]
mod tests;
