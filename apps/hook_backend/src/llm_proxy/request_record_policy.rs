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
            client: RequestRecordSidePolicy::new(RequestRecordSideConfig {
                level: snapshot.client_request_record_level,
                switches: RequestRecordSideSwitches {
                    request_headers: snapshot.client_record_request_headers,
                    request_body: snapshot.client_record_request_body,
                    response_headers: snapshot.client_record_response_headers,
                    response_body: snapshot.client_record_response_body,
                },
                max_request_body_size_kb: snapshot.client_max_request_body_size_kb,
                max_response_body_size_kb: snapshot.client_max_response_body_size_kb,
                sensitive_request_headers: &snapshot.client_sensitive_request_headers,
            })?,
            provider: RequestRecordSidePolicy::new(RequestRecordSideConfig {
                level: snapshot.provider_request_record_level,
                switches: RequestRecordSideSwitches {
                    request_headers: snapshot.provider_record_request_headers,
                    request_body: snapshot.provider_record_request_body,
                    response_headers: snapshot.provider_record_response_headers,
                    response_body: snapshot.provider_record_response_body,
                },
                max_request_body_size_kb: snapshot.provider_max_request_body_size_kb,
                max_response_body_size_kb: snapshot.provider_max_response_body_size_kb,
                sensitive_request_headers: &snapshot.provider_sensitive_request_headers,
            })?,
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
    pub(in crate::llm_proxy) record_response_headers: bool,
    pub(in crate::llm_proxy) record_response_body: bool,
    pub(in crate::llm_proxy) max_request_body_size_bytes: usize,
    pub(in crate::llm_proxy) max_response_body_size_bytes: usize,
    pub(in crate::llm_proxy) sensitive_request_headers: Vec<String>,
}

impl RequestRecordSidePolicy {
    fn new(config: RequestRecordSideConfig<'_>) -> Result<Self, String> {
        let level_switches = RequestRecordSideSwitches::from_level(config.level);
        let switches = level_switches.apply(config.switches);
        Ok(Self {
            level: config.level,
            record_request_headers: switches.request_headers,
            record_request_body: switches.request_body,
            record_response_headers: switches.response_headers,
            record_response_body: switches.response_body,
            max_request_body_size_bytes: size_kb_to_bytes(config.max_request_body_size_kb, "request record request body size")?,
            max_response_body_size_bytes: size_kb_to_bytes(config.max_response_body_size_kb, "request record response body size")?,
            sensitive_request_headers: sensitive_headers(config.sensitive_request_headers),
        })
    }

    pub(in crate::llm_proxy) fn should_record_request_headers(&self) -> bool {
        self.record_request_headers
    }

    pub(in crate::llm_proxy) fn should_record_request_body(&self) -> bool {
        self.record_request_body
    }

    pub(in crate::llm_proxy) fn should_record_response_headers(&self) -> bool {
        self.record_response_headers
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
struct RequestRecordSideSwitches {
    request_headers: bool,
    request_body: bool,
    response_headers: bool,
    response_body: bool,
}

impl RequestRecordSideSwitches {
    const fn from_level(level: RequestRecordLevel) -> Self {
        match level {
            RequestRecordLevel::Basic => Self::new(false, false, false, false),
            RequestRecordLevel::Headers => Self::new(true, false, true, false),
            RequestRecordLevel::Full => Self::new(true, true, true, true),
        }
    }

    const fn new(request_headers: bool, request_body: bool, response_headers: bool, response_body: bool) -> Self {
        Self {
            request_headers,
            request_body,
            response_headers,
            response_body,
        }
    }

    const fn apply(self, switches: Self) -> Self {
        Self {
            request_headers: self.request_headers && switches.request_headers,
            request_body: self.request_body && switches.request_body,
            response_headers: self.response_headers && switches.response_headers,
            response_body: self.response_body && switches.response_body,
        }
    }
}

struct RequestRecordSideConfig<'a> {
    level: RequestRecordLevel,
    switches: RequestRecordSideSwitches,
    max_request_body_size_kb: i64,
    max_response_body_size_kb: i64,
    sensitive_request_headers: &'a str,
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
mod tests;
