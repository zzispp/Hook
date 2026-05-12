mod selection;
mod url;

use rust_decimal::Decimal;
use types::api_token::ApiToken;
use types::model::TieredPricingConfig;

use super::{LlmProxyError, LlmProxyState};

#[derive(Clone, Debug)]
pub struct ProxyCandidate {
    pub trace: CandidateTrace,
    pub api_key: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub upstream_url: String,
    pub provider_model_name: String,
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: TieredPricingConfig,
    pub billing_multiplier: Decimal,
    pub max_retries: i32,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    pub cache_ttl_minutes: i32,
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
