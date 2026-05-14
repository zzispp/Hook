use axum::http::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Map, Value};

use crate::llm_proxy::request_record_policy::{RequestRecordPolicy, truncate_request_body};

const MASKED_HEADER_VALUE: &str = "****";

#[derive(Clone, Debug)]
pub(in crate::llm_proxy) struct RequestCapture {
    headers: HeaderMap,
    body: Value,
}

impl RequestCapture {
    pub(in crate::llm_proxy) fn new(headers: &HeaderMap, body: &Value) -> Self {
        Self {
            headers: headers.clone(),
            body: body.clone(),
        }
    }

    pub(in crate::llm_proxy) fn request_headers(&self, policy: &RequestRecordPolicy) -> Option<Value> {
        recorded_headers(&self.headers, policy)
    }

    pub(in crate::llm_proxy) fn request_body(&self, policy: &RequestRecordPolicy) -> Result<Option<Value>, serde_json::Error> {
        recorded_request_body(&self.body, policy)
    }
}

pub(in crate::llm_proxy) fn recorded_headers(headers: &HeaderMap, policy: &RequestRecordPolicy) -> Option<Value> {
    policy.should_record_request_headers().then(|| headers_value(headers, policy))
}

pub(in crate::llm_proxy) fn recorded_request_body(body: &Value, policy: &RequestRecordPolicy) -> Result<Option<Value>, serde_json::Error> {
    truncate_request_body(body, policy)
}

fn headers_value(headers: &HeaderMap, policy: &RequestRecordPolicy) -> Value {
    let mut output = Map::new();
    for (name, value) in headers {
        output.insert(name.as_str().to_owned(), header_value(name, value, policy));
    }
    Value::Object(output)
}

fn header_value(name: &HeaderName, value: &HeaderValue, policy: &RequestRecordPolicy) -> Value {
    if sensitive_header(name, policy) {
        return Value::String(MASKED_HEADER_VALUE.to_owned());
    }
    match value.to_str() {
        Ok(text) => Value::String(text.to_owned()),
        Err(_) => Value::Array(value.as_bytes().iter().copied().map(Value::from).collect()),
    }
}

fn sensitive_header(name: &HeaderName, policy: &RequestRecordPolicy) -> bool {
    policy
        .sensitive_request_headers
        .iter()
        .any(|candidate| name.as_str().eq_ignore_ascii_case(candidate))
}
