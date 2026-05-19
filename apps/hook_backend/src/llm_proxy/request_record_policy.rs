use serde_json::{Value, json};
use types::system_setting::RequestRecordLevel;

use crate::llm_proxy::cache::snapshot::SchedulingSnapshot;

const BYTES_PER_KB: i64 = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::llm_proxy) struct RequestRecordPolicies {
    client: RequestRecordSidePolicy,
    provider: RequestRecordSidePolicy,
}

impl RequestRecordPolicies {
    pub(in crate::llm_proxy) fn from_snapshot(snapshot: &SchedulingSnapshot) -> Result<Self, String> {
        Ok(Self {
            client: RequestRecordSidePolicy::new(
                snapshot.client_request_record_level,
                snapshot.client_max_request_body_size_kb,
                snapshot.client_max_response_body_size_kb,
                &snapshot.client_sensitive_request_headers,
            )?,
            provider: RequestRecordSidePolicy::new(
                snapshot.provider_request_record_level,
                snapshot.provider_max_request_body_size_kb,
                snapshot.provider_max_response_body_size_kb,
                &snapshot.provider_sensitive_request_headers,
            )?,
        })
    }

    pub(in crate::llm_proxy) const fn client(&self) -> &RequestRecordSidePolicy {
        &self.client
    }

    pub(in crate::llm_proxy) const fn provider(&self) -> &RequestRecordSidePolicy {
        &self.provider
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::llm_proxy) struct RequestRecordSidePolicy {
    level: RequestRecordLevel,
    pub(in crate::llm_proxy) record_request_headers: bool,
    pub(in crate::llm_proxy) record_request_body: bool,
    pub(in crate::llm_proxy) record_response_body: bool,
    pub(in crate::llm_proxy) max_request_body_size_bytes: usize,
    pub(in crate::llm_proxy) max_response_body_size_bytes: usize,
    pub(in crate::llm_proxy) sensitive_request_headers: Vec<String>,
}

impl RequestRecordSidePolicy {
    fn new(level: RequestRecordLevel, max_request_body_size_kb: i64, max_response_body_size_kb: i64, sensitive_request_headers: &str) -> Result<Self, String> {
        let switches = RequestRecordSwitches::from_level(level);
        Ok(Self {
            level,
            record_request_headers: switches.headers,
            record_request_body: switches.request_body,
            record_response_body: switches.response_body,
            max_request_body_size_bytes: size_kb_to_bytes(max_request_body_size_kb, "request record request body size")?,
            max_response_body_size_bytes: size_kb_to_bytes(max_response_body_size_kb, "request record response body size")?,
            sensitive_request_headers: sensitive_headers(sensitive_request_headers),
        })
    }

    pub(in crate::llm_proxy) fn should_record_request_headers(&self) -> bool {
        self.record_request_headers
    }

    pub(in crate::llm_proxy) fn should_record_request_body(&self) -> bool {
        self.record_request_body
    }

    pub(in crate::llm_proxy) fn should_record_response_body(&self) -> bool {
        self.record_response_body
    }

    pub(in crate::llm_proxy) fn response_body(&self, body: Option<Value>) -> Result<Option<Value>, serde_json::Error> {
        if !self.should_record_response_body() {
            return Ok(None);
        }
        body.map(|value| truncate_body(value, self.max_response_body_size_bytes)).transpose()
    }
}

#[derive(Clone, Copy)]
struct RequestRecordSwitches {
    headers: bool,
    request_body: bool,
    response_body: bool,
}

impl RequestRecordSwitches {
    const fn from_level(level: RequestRecordLevel) -> Self {
        match level {
            RequestRecordLevel::Basic => Self::new(false, false, false),
            RequestRecordLevel::Headers => Self::new(true, false, false),
            RequestRecordLevel::Full => Self::new(true, true, true),
        }
    }

    const fn new(headers: bool, request_body: bool, response_body: bool) -> Self {
        Self {
            headers,
            request_body,
            response_body,
        }
    }
}

pub(in crate::llm_proxy) fn truncate_request_body(body: &Value, policy: &RequestRecordSidePolicy) -> Result<Option<Value>, serde_json::Error> {
    if !policy.should_record_request_body() {
        return Ok(None);
    }
    truncate_body(body.clone(), policy.max_request_body_size_bytes).map(Some)
}

fn size_kb_to_bytes(value: i64, field: &str) -> Result<usize, String> {
    let bytes = value.checked_mul(BYTES_PER_KB).ok_or_else(|| format!("{field} overflows byte conversion"))?;
    usize::try_from(bytes).map_err(|_| format!("{field} does not fit this platform"))
}

fn sensitive_headers(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| item.trim().to_ascii_lowercase())
        .filter(|item| !item.is_empty())
        .collect()
}

fn truncate_body(body: Value, max_size_bytes: usize) -> Result<Value, serde_json::Error> {
    let text = serde_json::to_string(&body)?;
    if text.len() <= max_size_bytes {
        return Ok(body);
    }
    Ok(json!({
        "_truncated": true,
        "_original_size": text.len(),
        "_max_size": max_size_bytes,
        "_content": truncate_text(&text, max_size_bytes),
    }))
}

fn truncate_text(text: &str, max_size_bytes: usize) -> &str {
    let mut end = max_size_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[cfg(test)]
mod tests {
    use types::provider::ProviderSchedulingMode;

    use super::*;
    use crate::llm_proxy::proxy::capture::RequestCapture;
    use axum::http::{HeaderMap, header};
    use serde_json::json;

    #[test]
    fn request_capture_applies_header_switch_and_sensitive_header_redaction() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, "sk-test-secret".parse().unwrap());
        headers.insert("x-trace-id", "trace-1".parse().unwrap());
        let policy = RequestRecordSidePolicy {
            level: RequestRecordLevel::Headers,
            record_request_headers: true,
            record_request_body: false,
            record_response_body: false,
            max_request_body_size_bytes: 1024,
            max_response_body_size_bytes: 1024,
            sensitive_request_headers: vec!["authorization".into()],
        };

        let capture = RequestCapture::new(&headers, &json!({"model": "gpt-5.5"}));

        assert_eq!(
            capture.request_headers(&policy),
            Some(json!({"authorization": "****", "x-trace-id": "trace-1"}))
        );
        assert_eq!(capture.request_body(&policy).unwrap(), None);
    }

    #[test]
    fn request_capture_truncates_enabled_request_body() {
        let headers = HeaderMap::new();
        let policy = RequestRecordSidePolicy {
            level: RequestRecordLevel::Full,
            record_request_headers: false,
            record_request_body: true,
            record_response_body: true,
            max_request_body_size_bytes: 32,
            max_response_body_size_bytes: 32,
            sensitive_request_headers: vec![],
        };
        let body = json!({"prompt": "abcdefghijklmnopqrstuvwxyz"});

        let capture = RequestCapture::new(&headers, &body);
        let request_body = capture.request_body(&policy).unwrap().unwrap();
        let response_body = policy.response_body(Some(body)).unwrap().unwrap();

        assert_eq!(request_body.get("_truncated").and_then(Value::as_bool), Some(true));
        assert_eq!(response_body.get("_truncated").and_then(Value::as_bool), Some(true));
    }

    #[test]
    fn request_record_policy_can_be_restored_from_runtime_snapshot() {
        let snapshot = SchedulingSnapshot {
            default_rate_limit_rpm: 0,
            scheduling_mode: ProviderSchedulingMode::FixedOrder,
            cache_affinity_ttl_minutes: 5,
            models: Vec::new(),
            groups: Vec::new(),
            users: Vec::new(),
            providers: Vec::new(),
            client_request_record_level: RequestRecordLevel::Headers,
            client_max_request_body_size_kb: 12,
            client_max_response_body_size_kb: 34,
            client_sensitive_request_headers: "authorization, x-api-key".into(),
            provider_request_record_level: RequestRecordLevel::Full,
            provider_max_request_body_size_kb: 56,
            provider_max_response_body_size_kb: 78,
            provider_sensitive_request_headers: "x-provider-key".into(),
            provider_cooldown_policy: Default::default(),
        };

        let policies = RequestRecordPolicies::from_snapshot(&snapshot).unwrap();

        assert_eq!(
            policies.client,
            RequestRecordSidePolicy {
                level: RequestRecordLevel::Headers,
                record_request_headers: true,
                record_request_body: false,
                record_response_body: false,
                max_request_body_size_bytes: 12 * 1024,
                max_response_body_size_bytes: 34 * 1024,
                sensitive_request_headers: vec!["authorization".into(), "x-api-key".into()],
            }
        );
        assert_eq!(
            policies.provider,
            RequestRecordSidePolicy {
                level: RequestRecordLevel::Full,
                record_request_headers: true,
                record_request_body: true,
                record_response_body: true,
                max_request_body_size_bytes: 56 * 1024,
                max_response_body_size_bytes: 78 * 1024,
                sensitive_request_headers: vec!["x-provider-key".into()],
            }
        );
    }
}
