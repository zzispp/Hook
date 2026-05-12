use axum::http::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{Map, Value};

const MASKED_HEADER_VALUE: &str = "****";
const SENSITIVE_HEADERS: &[&str] = &["authorization", "proxy-authorization", "x-api-key", "api-key", "cookie", "set-cookie"];

#[derive(Clone, Debug)]
pub(in crate::llm_proxy) struct RequestCapture {
    pub(in crate::llm_proxy) request_headers: Value,
    pub(in crate::llm_proxy) request_body: Value,
}

impl RequestCapture {
    pub(in crate::llm_proxy) fn new(headers: &HeaderMap, body: &Value) -> Self {
        Self {
            request_headers: headers_value(headers),
            request_body: body.clone(),
        }
    }
}

fn headers_value(headers: &HeaderMap) -> Value {
    let mut output = Map::new();
    for (name, value) in headers {
        output.insert(name.as_str().to_owned(), header_value(name, value));
    }
    Value::Object(output)
}

fn header_value(name: &HeaderName, value: &HeaderValue) -> Value {
    if sensitive_header(name) {
        return Value::String(MASKED_HEADER_VALUE.to_owned());
    }
    match value.to_str() {
        Ok(text) => Value::String(text.to_owned()),
        Err(_) => Value::Array(value.as_bytes().iter().copied().map(Value::from).collect()),
    }
}

fn sensitive_header(name: &HeaderName) -> bool {
    SENSITIVE_HEADERS.iter().any(|candidate| name.as_str().eq_ignore_ascii_case(candidate))
}
