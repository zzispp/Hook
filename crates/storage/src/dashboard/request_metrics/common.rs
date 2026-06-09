use crate::provider::record::request_records;

use super::constants::{STATUS_CANCELLED, STATUS_FAILED, STATUS_PENDING, STATUS_STREAMING, STATUS_SUCCESS};

pub(super) fn clean_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}

pub(super) fn positive(value: Option<i64>) -> i64 {
    value.unwrap_or_default().max(0)
}

pub(super) fn is_terminal_status(status: &str) -> bool {
    status == STATUS_SUCCESS || status == STATUS_FAILED || status == STATUS_CANCELLED
}

pub(super) fn is_failed_status(status: &str) -> bool {
    status == STATUS_FAILED || status == STATUS_CANCELLED
}

pub(super) fn is_active_status(status: &str) -> bool {
    status == STATUS_PENDING || status == STATUS_STREAMING
}

pub(super) fn request_error_category(record: &request_records::Model) -> Option<String> {
    if !is_failed_status(&record.status) {
        return None;
    }
    Some(match clean_optional(record.client_error_type.clone()) {
        Some(value) => value,
        None if record.client_status_code.is_some_and(|status| status >= 500) => "server_error".into(),
        None if record.client_status_code == Some(429) => "rate_limit".into(),
        None if record.client_status_code.is_some_and(|status| status >= 400) => "client_error".into(),
        None => clean_optional(record.termination_reason.clone()).unwrap_or_else(|| "unknown".into()),
    })
}

pub(super) fn request_is_timeout(record: &request_records::Model) -> bool {
    record.termination_reason.as_deref().is_some_and(|value| value.contains("timeout"))
        || record.client_error_type.as_deref().is_some_and(|value| value.contains("timeout"))
}

pub(super) fn request_is_quota_limited(record: &request_records::Model) -> bool {
    record.client_error_type.as_deref() == Some("hook_api_error") || record.client_error_message.as_deref().is_some_and(|value| value.contains("quota"))
}
