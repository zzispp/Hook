use proxy::format_conversion::ApiFormat;
use req::{HeaderMap, Request, RequestBuilder};
use serde_json::Value;

use super::{LlmProxyError, header_rules::apply_provider_header_rules, timeout::non_stream_total_timeout};
use crate::llm_proxy::{
    candidate::ProxyCandidate,
    formats::{self, AuthScheme},
};

const ANTHROPIC_VERSION: &str = "2023-06-01";

pub(super) enum UpstreamRequestBody<'a> {
    Json(&'a Value),
    Multipart(req::multipart::Form),
}

pub(super) struct UpstreamRequestInput<'a> {
    pub(super) candidate: &'a ProxyCandidate,
    pub(super) target_format: ApiFormat,
    pub(super) body: UpstreamRequestBody<'a>,
    pub(super) current_body: &'a Value,
    pub(super) original_body: &'a Value,
    pub(super) provider_headers: &'a HeaderMap,
    pub(super) is_stream: bool,
}

pub(super) fn upstream_request(client: &req::ReqwestClient, input: UpstreamRequestInput<'_>) -> Result<Request, LlmProxyError> {
    let builder = request_builder(client, input.candidate, input.body);
    let metadata = formats::endpoint_metadata(
        &input.candidate.trace.provider_api_format,
        input.current_body.get("stream").and_then(Value::as_bool).unwrap_or(false),
    )?;
    if input.target_format != metadata.data_format {
        return Err(LlmProxyError::InvalidRequest(format!(
            "provider format metadata mismatch: {}",
            input.candidate.trace.provider_api_format
        )));
    }
    let builder = apply_extra_headers(apply_auth(builder, input.candidate, metadata.auth_scheme), input.provider_headers);
    let mut request = client.build_request(apply_timeout(builder, input.candidate, input.is_stream))?;
    apply_provider_header_rules(request.headers_mut(), &input.candidate.header_rules, input.current_body, input.original_body)?;
    Ok(request)
}

fn request_builder(client: &req::ReqwestClient, candidate: &ProxyCandidate, body: UpstreamRequestBody<'_>) -> RequestBuilder {
    match body {
        UpstreamRequestBody::Json(body) => client.post(candidate.upstream_url.clone()).json(body),
        UpstreamRequestBody::Multipart(form) => client.post(candidate.upstream_url.clone()).multipart(form),
    }
}

fn apply_extra_headers(mut builder: RequestBuilder, headers: &HeaderMap) -> RequestBuilder {
    for (name, value) in headers {
        builder = builder.header(name, value);
    }
    builder
}

fn apply_auth(builder: RequestBuilder, candidate: &ProxyCandidate, scheme: AuthScheme) -> RequestBuilder {
    match scheme {
        AuthScheme::Bearer => builder.bearer_auth(candidate.api_key.as_str()),
        AuthScheme::Anthropic => builder
            .header("x-api-key", candidate.api_key.as_str())
            .header("anthropic-version", ANTHROPIC_VERSION),
        AuthScheme::Gemini => builder.header("x-goog-api-key", candidate.api_key.as_str()),
    }
}

fn apply_timeout(builder: RequestBuilder, candidate: &ProxyCandidate, is_stream: bool) -> RequestBuilder {
    match non_stream_total_timeout(candidate, is_stream) {
        Some(timeout) => builder.timeout(timeout),
        None => builder,
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::model::TieredPricingConfig;

    use super::apply_timeout;
    use crate::llm_proxy::candidate::{CandidateRoute, CandidateTrace, ProxyCandidate};

    #[test]
    fn stream_request_does_not_set_total_request_timeout() {
        let client = req::ReqwestClient::from_builder(req::long_stream_builder()).unwrap();
        let request = client
            .build_request(apply_timeout(client.post("https://example.com".into()), &candidate(), true))
            .unwrap();

        assert_eq!(request.timeout(), None);
    }

    #[test]
    fn non_stream_request_uses_default_total_timeout_when_provider_timeout_is_missing() {
        let client = req::ReqwestClient::from_builder(req::long_stream_builder()).unwrap();
        let request = client
            .build_request(apply_timeout(client.post("https://example.com".into()), &candidate(), false))
            .unwrap();
        let expected = req::default_timeout();

        assert_eq!(request.timeout(), Some(&expected));
    }

    #[test]
    fn non_stream_request_uses_provider_total_timeout() {
        let client = req::ReqwestClient::from_builder(req::long_stream_builder()).unwrap();
        let mut candidate = candidate();
        candidate.request_timeout_seconds = Some(120.0);
        let request = client
            .build_request(apply_timeout(client.post("https://example.com".into()), &candidate, false))
            .unwrap();

        assert_eq!(request.timeout(), Some(&std::time::Duration::from_secs(120)));
    }

    fn candidate() -> ProxyCandidate {
        ProxyCandidate {
            trace: trace(),
            requested_model_name: "gpt-5.5".into(),
            api_key: "secret".into(),
            base_url: "https://example.com".into(),
            custom_path: None,
            upstream_url: "https://example.com/v1/responses".into(),
            provider_model_name: "gpt-5.5".into(),
            reasoning_effort: None,
            header_rules: None,
            body_rules: None,
            format_acceptance_config: None,
            key_supports_image_generation: false,
            price_per_request: None,
            tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
            billing_multiplier: Decimal::ONE,
            max_retries: 0,
            request_timeout_seconds: None,
            stream_first_byte_timeout_seconds: Some(30.0),
            stream_first_output_timeout_seconds: Some(45.0),
            stream_idle_timeout_seconds: None,
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
            is_cached: false,
            route: CandidateRoute { options: Vec::new() },
        }
    }

    fn trace() -> CandidateTrace {
        CandidateTrace {
            token_id: Some("token-1".into()),
            user_id_snapshot: Some("user-1".into()),
            username_snapshot: Some("alice".into()),
            token_name_snapshot: Some("token".into()),
            token_prefix_snapshot: Some("sk-test".into()),
            group_code: Some("default".into()),
            global_model_id: "model-1".into(),
            provider_model_id: "provider-model-1".into(),
            model_name_snapshot: "gpt-5.5".into(),
            provider_id: "provider-1".into(),
            provider_name_snapshot: "Provider".into(),
            endpoint_id: "endpoint-1".into(),
            endpoint_name_snapshot: "endpoint".into(),
            key_id: "key-1".into(),
            key_name_snapshot: "Key".into(),
            key_preview_snapshot: "***test".into(),
            client_api_format: "openai:cli".into(),
            provider_api_format: "openai:cli".into(),
            needs_conversion: false,
            is_stream: true,
            is_cached: false,
            routing_profile_id: types::provider::RoutingProfileId::Balanced,
            routing_profile_ema_alpha: types::provider::default_ema_alpha(),
            routing_context_key: "group=default|model=model-1|format=openai:cli|stream=true|size=unknown|cap=none".into(),
            route_config_fingerprint: "route-fingerprint".into(),
            price_config_fingerprint: "price-fingerprint".into(),
            candidate_index: 0,
        }
    }
}
