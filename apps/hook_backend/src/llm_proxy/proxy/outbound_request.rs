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
    let mut request = client.build_request(apply_timeout(builder, candidate))?;
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

fn apply_timeout(builder: RequestBuilder, candidate: &ProxyCandidate) -> RequestBuilder {
    match proxy_timeouts(candidate).request {
        Some(timeout) => builder.timeout(timeout),
        None => builder,
    }
}
