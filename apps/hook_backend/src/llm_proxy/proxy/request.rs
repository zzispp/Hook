use proxy::format_conversion::{ApiFormat, FormatConversionRegistry};
use serde_json::{Map, Value};
use types::api_token::ApiToken;

use crate::llm_proxy::{
    LlmProxyError, LlmProxyState,
    audit::record_scheduled_candidates,
    candidate::{CandidateRequest, ProxyCandidate, select_candidates},
    formats, rate_limit,
};

use super::{body_rules::apply_provider_body_rules, capture::RequestCapture};

pub(super) struct PreparedProxyRequest {
    pub(super) request_id: String,
    pub(super) candidates: Vec<ProxyCandidate>,
    pub(super) body: Value,
    pub(super) is_stream: bool,
    pub(super) force_non_stream: bool,
}

pub(super) struct AttemptPayload {
    pub(super) body: Value,
    pub(super) original_body: Value,
    pub(super) source_format: ApiFormat,
    pub(super) target_format: ApiFormat,
}

pub(super) async fn prepare_proxy_request(
    state: &LlmProxyState,
    token: &ApiToken,
    body: Value,
    api_format: &'static str,
    force_non_stream: bool,
    capture: RequestCapture,
) -> Result<PreparedProxyRequest, LlmProxyError> {
    let model_name = required_model(&body)?;
    rate_limit::enforce_request_limits(state, token).await?;
    let is_stream = is_streaming(&body) && !force_non_stream;
    let selection = select_candidates(
        state,
        token,
        CandidateRequest {
            api_format,
            model_name,
            is_stream,
        },
    )
    .await?;
    record_scheduled_candidates(state, &selection, &capture).await?;
    Ok(PreparedProxyRequest {
        request_id: selection.request_id,
        candidates: selection.candidates,
        body,
        is_stream,
        force_non_stream,
    })
}

pub(super) fn attempt_payload(body: Value, candidate: &ProxyCandidate, force_non_stream: bool) -> Result<AttemptPayload, LlmProxyError> {
    let original_body = body.clone();
    let (body, source_format, target_format) = upstream_body(body, &original_body, candidate, force_non_stream)?;
    Ok(AttemptPayload {
        body,
        original_body,
        source_format,
        target_format,
    })
}

fn required_model(body: &Value) -> Result<&str, LlmProxyError> {
    body.get("model")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must include a non-empty model".into()))
}

fn is_streaming(body: &Value) -> bool {
    body.get("stream").and_then(Value::as_bool).unwrap_or(false)
}

fn upstream_body(
    body: Value,
    original_body: &Value,
    candidate: &ProxyCandidate,
    force_non_stream: bool,
) -> Result<(Value, ApiFormat, ApiFormat), LlmProxyError> {
    let mut body = body;
    let source = formats::parse_api_format(candidate.trace.client_api_format.as_str())?;
    let target = formats::parse_api_format(candidate.trace.provider_api_format.as_str())?;
    if candidate.trace.needs_conversion {
        body = FormatConversionRegistry::default()
            .convert_request(&body, source, target)
            .map_err(|error| LlmProxyError::InvalidRequest(error.to_string()))?;
    }
    rewrite_upstream_body(&mut body, candidate, force_non_stream, target)?;
    apply_reasoning_effort(&mut body, candidate, target)?;
    apply_provider_body_rules(&mut body, &candidate.body_rules, original_body)?;
    Ok((body, source, target))
}

fn rewrite_upstream_body(body: &mut Value, candidate: &ProxyCandidate, force_non_stream: bool, target: ApiFormat) -> Result<(), LlmProxyError> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    object.insert("model".into(), Value::String(candidate.provider_model_name.clone()));
    if force_non_stream {
        object.remove("stream");
    }
    if target == ApiFormat::GeminiChat {
        object.remove("model");
        object.remove("stream");
    }
    Ok(())
}

fn apply_reasoning_effort(body: &mut Value, candidate: &ProxyCandidate, target: ApiFormat) -> Result<(), LlmProxyError> {
    let Some(reasoning_effort) = candidate.reasoning_effort.as_deref() else {
        return Ok(());
    };
    let object = body
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request body must be a JSON object".into()))?;
    match target {
        ApiFormat::OpenAiChat => {
            object.insert("reasoning_effort".into(), Value::String(reasoning_effort.to_owned()));
            Ok(())
        }
        ApiFormat::OpenAiResponses => {
            reasoning_object(object)?.insert("effort".into(), Value::String(reasoning_effort.to_owned()));
            Ok(())
        }
        _ => Err(LlmProxyError::InvalidRequest(format!(
            "reasoning_effort override is not supported for provider format {}",
            candidate.trace.provider_api_format
        ))),
    }
}

fn reasoning_object(object: &mut Map<String, Value>) -> Result<&mut Map<String, Value>, LlmProxyError> {
    let value = object.entry("reasoning").or_insert_with(|| Value::Object(Map::new()));
    value
        .as_object_mut()
        .ok_or_else(|| LlmProxyError::InvalidRequest("request field reasoning must be a JSON object".into()))
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use serde_json::json;
    use types::model::TieredPricingConfig;

    use super::apply_reasoning_effort;
    use crate::llm_proxy::candidate::{CandidateRoute, CandidateTrace, ProxyCandidate};
    use proxy::format_conversion::ApiFormat;

    #[test]
    fn reasoning_effort_override_sets_openai_chat_field() {
        let mut body = json!({"model": "gpt-5.5"});

        apply_reasoning_effort(&mut body, &candidate("openai_chat"), ApiFormat::OpenAiChat).unwrap();

        assert_eq!(body["reasoning_effort"], "high");
    }

    #[test]
    fn reasoning_effort_override_sets_openai_responses_nested_field() {
        let mut body = json!({"model": "gpt-5.5"});

        apply_reasoning_effort(&mut body, &candidate("openai_cli"), ApiFormat::OpenAiResponses).unwrap();

        assert_eq!(body["reasoning"]["effort"], "high");
    }

    fn candidate(provider_api_format: &str) -> ProxyCandidate {
        ProxyCandidate {
            trace: CandidateTrace {
                token_id: Some("token-1".into()),
                user_id_snapshot: Some("user-1".into()),
                username_snapshot: Some("alice".into()),
                token_name_snapshot: Some("token".into()),
                token_prefix_snapshot: Some("sk-test".into()),
                group_code: Some("default".into()),
                global_model_id: "model-1".into(),
                model_name_snapshot: "gpt-5.5".into(),
                provider_id: "provider-1".into(),
                provider_name_snapshot: "Provider".into(),
                endpoint_id: "endpoint-1".into(),
                endpoint_name_snapshot: provider_api_format.into(),
                key_id: "key-1".into(),
                key_name_snapshot: "Key".into(),
                key_preview_snapshot: "***test".into(),
                client_api_format: "openai_chat".into(),
                provider_api_format: provider_api_format.into(),
                needs_conversion: false,
                is_stream: false,
                candidate_index: 0,
            },
            requested_model_name: "gpt-5.5".into(),
            api_key: "secret".into(),
            base_url: "https://example.com".into(),
            custom_path: None,
            upstream_url: "https://example.com/v1/chat/completions".into(),
            provider_model_name: "upstream-model".into(),
            reasoning_effort: Some("high".into()),
            header_rules: None,
            body_rules: None,
            price_per_request: None,
            tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
            billing_multiplier: Decimal::ONE,
            max_retries: 0,
            request_timeout_seconds: None,
            stream_first_byte_timeout_seconds: None,
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
            route: CandidateRoute {
                endpoints: Vec::new(),
                keys: Vec::new(),
            },
        }
    }
}
