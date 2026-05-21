use axum::http::{HeaderMap, header};
use serde_json::{Value, json};
use types::{provider::ProviderSchedulingMode, system_setting::RequestRecordLevel};

use super::{RequestRecordPolicies, RequestRecordSidePolicy};
use crate::llm_proxy::{cache::snapshot::SchedulingSnapshot, proxy::capture::RequestCapture};

#[test]
fn request_capture_applies_header_switch_and_sensitive_header_redaction() {
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, "sk-test-secret".parse().unwrap());
    headers.insert("x-trace-id", "trace-1".parse().unwrap());
    let policy = policy(RequestRecordLevel::Headers, true, false, false, false, vec!["authorization".into()]);

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
    let policy = policy(RequestRecordLevel::Full, false, true, false, true, Vec::new());
    let body = json!({"prompt": "abcdefghijklmnopqrstuvwxyz"});

    let capture = RequestCapture::new(&headers, &body);
    let request_body = capture.request_body(&policy).unwrap().unwrap();
    let response_body = policy.response_body(Some(body)).unwrap().unwrap();

    assert_eq!(request_body.get("_truncated").and_then(Value::as_bool), Some(true));
    assert_eq!(response_body.get("_truncated").and_then(Value::as_bool), Some(true));
}

#[test]
fn request_record_policy_can_be_restored_from_runtime_snapshot() {
    let mut snapshot = snapshot();
    snapshot.client_request_record_level = RequestRecordLevel::Headers;
    snapshot.provider_request_record_level = RequestRecordLevel::Full;

    let policies = RequestRecordPolicies::from_snapshot(&snapshot).unwrap();

    assert_eq!(
        policies.client,
        RequestRecordSidePolicy {
            level: RequestRecordLevel::Headers,
            record_request_headers: true,
            record_request_body: false,
            record_response_headers: true,
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
            record_response_headers: true,
            record_response_body: true,
            max_request_body_size_bytes: 56 * 1024,
            max_response_body_size_bytes: 78 * 1024,
            sensitive_request_headers: vec!["x-provider-key".into()],
        }
    );
}

#[test]
fn runtime_snapshot_switches_control_payload_parts_independently() {
    let mut snapshot = snapshot();
    snapshot.client_request_record_level = RequestRecordLevel::Full;
    snapshot.client_record_request_body = false;
    snapshot.client_record_response_body = false;
    snapshot.provider_request_record_level = RequestRecordLevel::Full;
    snapshot.provider_record_request_headers = false;
    snapshot.provider_record_response_headers = false;

    let policies = RequestRecordPolicies::from_snapshot(&snapshot).unwrap();

    assert!(policies.client.should_record_request_headers());
    assert!(!policies.client.should_record_request_body());
    assert!(policies.client.should_record_response_headers());
    assert!(!policies.client.should_record_response_body());
    assert!(!policies.provider.should_record_request_headers());
    assert!(policies.provider.should_record_request_body());
    assert!(!policies.provider.should_record_response_headers());
    assert!(policies.provider.should_record_response_body());
}

#[test]
fn request_record_level_still_limits_enabled_switches() {
    let policies = RequestRecordPolicies::from_snapshot(&snapshot()).unwrap();

    assert!(policies.client.should_record_request_headers());
    assert!(!policies.client.should_record_request_body());
    assert!(policies.client.should_record_response_headers());
    assert!(!policies.client.should_record_response_body());
}

fn policy(
    level: RequestRecordLevel,
    record_request_headers: bool,
    record_request_body: bool,
    record_response_headers: bool,
    record_response_body: bool,
    sensitive_request_headers: Vec<String>,
) -> RequestRecordSidePolicy {
    RequestRecordSidePolicy {
        level,
        record_request_headers,
        record_request_body,
        record_response_headers,
        record_response_body,
        max_request_body_size_bytes: 32,
        max_response_body_size_bytes: 32,
        sensitive_request_headers,
    }
}

fn snapshot() -> SchedulingSnapshot {
    SchedulingSnapshot {
        default_rate_limit_rpm: 0,
        scheduling_mode: ProviderSchedulingMode::FixedOrder,
        cache_affinity_ttl_minutes: 5,
        client_request_record_level: RequestRecordLevel::Headers,
        client_record_request_headers: true,
        client_record_request_body: true,
        client_record_response_headers: true,
        client_record_response_body: true,
        client_max_request_body_size_kb: 12,
        client_max_response_body_size_kb: 34,
        client_sensitive_request_headers: "authorization, x-api-key".into(),
        provider_request_record_level: RequestRecordLevel::Full,
        provider_record_request_headers: true,
        provider_record_request_body: true,
        provider_record_response_headers: true,
        provider_record_response_body: true,
        provider_max_request_body_size_kb: 56,
        provider_max_response_body_size_kb: 78,
        provider_sensitive_request_headers: "x-provider-key".into(),
        provider_cooldown_policy: Default::default(),
        models: Vec::new(),
        groups: Vec::new(),
        users: Vec::new(),
        providers: Vec::new(),
    }
}
