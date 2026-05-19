use proxy::format_conversion::ApiFormat;
use req::{HeaderMap, Request, RequestBuilder};
use serde_json::Value;

use super::{LlmProxyError, header_rules::apply_provider_header_rules, timeout::proxy_timeouts};
use crate::llm_proxy::{
    candidate::ProxyCandidate,
    formats::{self, AuthScheme},
};

const ANTHROPIC_VERSION: &str = "2023-06-01";

pub(super) fn upstream_request(
    client: &req::ReqwestClient,
    candidate: &ProxyCandidate,
    target_format: ApiFormat,
    body: &Value,
    original_body: &Value,
    provider_headers: &HeaderMap,
    is_stream: bool,
) -> Result<Request, LlmProxyError> {
    let builder = client.post(candidate.upstream_url.clone()).json(body);
    let metadata = formats::endpoint_metadata(
        &candidate.trace.provider_api_format,
        body.get("stream").and_then(Value::as_bool).unwrap_or(false),
    )?;
    if target_format != metadata.data_format {
        return Err(LlmProxyError::InvalidRequest(format!(
            "provider format metadata mismatch: {}",
            candidate.trace.provider_api_format
        )));
    }
    let builder = apply_extra_headers(apply_auth(builder, candidate, metadata.auth_scheme), provider_headers);
    let mut request = client.build_request(apply_timeout(builder, candidate, is_stream))?;
    apply_provider_header_rules(request.headers_mut(), &candidate.header_rules, body, original_body)?;
    Ok(request)
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
    if is_stream {
        return builder;
    }
    match proxy_timeouts(candidate).request {
        Some(timeout) => builder.timeout(timeout),
        None => builder.timeout(req::default_timeout()),
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
            price_per_request: None,
            tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
            billing_multiplier: Decimal::ONE,
            max_retries: 0,
            request_timeout_seconds: None,
            stream_first_byte_timeout_seconds: Some(30.0),
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
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
            client_api_format: "openai_cli".into(),
            provider_api_format: "openai_cli".into(),
            needs_conversion: false,
            is_stream: true,
            candidate_index: 0,
        }
    }
}
