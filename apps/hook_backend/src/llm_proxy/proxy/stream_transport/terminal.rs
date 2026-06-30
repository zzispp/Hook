use axum::http::StatusCode;
use serde_json::json;
use types::model::PatchField;

use super::{
    body_capture::StreamResponseBodyPatches,
    status::{StreamEndReason, StreamStatus},
};

const BAD_GATEWAY: i32 = 502;
const GATEWAY_TIMEOUT: i32 = 504;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StreamClientFailure {
    pub(super) status: StatusCode,
    pub(super) error_type: &'static str,
    pub(super) message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StreamCooldownFailure {
    pub(super) status_code: i32,
    pub(super) error_type: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct StreamTerminalObservability {
    pub(super) response_headers_time_ms: Option<i64>,
    pub(super) first_sse_event_time_ms: Option<i64>,
    pub(super) first_token_time_ms: Option<i64>,
    pub(super) first_byte_time_ms: Option<i64>,
    pub(super) latency_ms: i64,
    pub(super) bodies: StreamResponseBodyPatches,
    pub(super) provider_frame_count: usize,
    pub(super) client_frame_count: usize,
    pub(super) received_response_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StreamTerminalSummary {
    pub(super) record_status: &'static str,
    pub(super) status_code: i32,
    pub(super) error_type: Option<&'static str>,
    pub(super) error_message: Option<String>,
    pub(super) termination_origin: PatchField<String>,
    pub(super) termination_reason: PatchField<String>,
    pub(super) stream_end_reason: PatchField<String>,
    pub(super) cooldown: Option<StreamCooldownFailure>,
    pub(super) observability: StreamTerminalObservability,
}

impl StreamTerminalSummary {
    pub(super) fn success(upstream_status: StatusCode, status: &StreamStatus) -> Self {
        Self {
            record_status: "success",
            status_code: upstream_status.as_u16() as i32,
            error_type: None,
            error_message: None,
            termination_origin: PatchField::Null,
            termination_reason: PatchField::Null,
            stream_end_reason: status.stream_end_reason_patch(),
            cooldown: None,
            observability: StreamTerminalObservability::default(),
        }
    }

    pub(super) fn provider_failure(error_type: &'static str, error_message: impl Into<String>, status: &StreamStatus) -> Self {
        let error_message = error_message.into();
        Self {
            record_status: "failed",
            status_code: failure_status_code(error_type),
            error_type: Some(error_type),
            error_message: Some(error_message),
            termination_origin: PatchField::Null,
            termination_reason: PatchField::Null,
            stream_end_reason: status.stream_end_reason_patch(),
            cooldown: cooldown_failure(error_type),
            observability: StreamTerminalObservability::default(),
        }
    }

    pub(super) fn incomplete(status: &StreamStatus) -> Self {
        Self {
            record_status: "failed",
            status_code: BAD_GATEWAY,
            error_type: Some("upstream_incomplete_stream"),
            error_message: Some("upstream stream ended without a terminal event".into()),
            termination_origin: PatchField::Null,
            termination_reason: PatchField::Null,
            stream_end_reason: status.stream_end_reason_patch(),
            cooldown: Some(StreamCooldownFailure {
                status_code: BAD_GATEWAY,
                error_type: StreamEndReason::UpstreamEofWithoutCompletion.as_str(),
            }),
            observability: StreamTerminalObservability::default(),
        }
    }

    pub(super) fn with_observability(mut self, observability: StreamTerminalObservability) -> Self {
        self.observability = observability;
        self
    }

    pub(super) fn client_failure(&self) -> Option<StreamClientFailure> {
        let error_type = self.error_type?;
        Some(StreamClientFailure {
            status: StatusCode::from_u16(self.status_code.try_into().ok()?).ok()?,
            error_type,
            message: self.error_message.clone().unwrap_or_else(|| "upstream stream failed".into()),
        })
    }
}

impl StreamClientFailure {
    pub(super) fn body(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&json!({
            "error": {
                "message": self.message,
                "type": self.error_type,
                "code": self.status.as_u16()
            }
        }))
    }
}

fn failure_status_code(error_type: &str) -> i32 {
    match error_type {
        "first_byte_timeout" | "first_token_timeout" | "stream_idle_timeout" | "upstream_timeout" => GATEWAY_TIMEOUT,
        _ => BAD_GATEWAY,
    }
}

fn cooldown_failure(error_type: &'static str) -> Option<StreamCooldownFailure> {
    let status_code = match error_type {
        "first_byte_timeout" | "first_token_timeout" | "stream_idle_timeout" | "upstream_timeout" => GATEWAY_TIMEOUT,
        "upstream_response_read_error" => BAD_GATEWAY,
        _ => return None,
    };
    Some(StreamCooldownFailure { status_code, error_type })
}

#[cfg(test)]
mod tests {
    use super::{StreamCooldownFailure, StreamTerminalSummary, failure_status_code};
    use crate::llm_proxy::proxy::stream_transport::status::{StreamEndReason, StreamStatus};

    #[test]
    fn incomplete_stream_is_failed_bad_gateway() {
        let mut status = StreamStatus::default();
        status.set_end_reason(StreamEndReason::UpstreamEofWithoutCompletion, None);

        let summary = StreamTerminalSummary::incomplete(&status);

        assert_eq!(summary.record_status, "failed");
        assert_eq!(summary.status_code, 502);
        assert_eq!(summary.error_type, Some("upstream_incomplete_stream"));
        assert_eq!(summary.cooldown.expect("cooldown").error_type, "upstream_eof_without_completion");
    }

    #[test]
    fn timeout_failures_use_gateway_timeout() {
        assert_eq!(failure_status_code("first_byte_timeout"), 504);
        assert_eq!(failure_status_code("first_token_timeout"), 504);
        assert_eq!(failure_status_code("stream_idle_timeout"), 504);
    }

    #[test]
    fn stream_provider_failures_enter_cooldown_with_real_error_type() {
        let cases = [
            (
                "first_byte_timeout",
                StreamCooldownFailure {
                    status_code: 504,
                    error_type: "first_byte_timeout",
                },
            ),
            (
                "first_token_timeout",
                StreamCooldownFailure {
                    status_code: 504,
                    error_type: "first_token_timeout",
                },
            ),
            (
                "stream_idle_timeout",
                StreamCooldownFailure {
                    status_code: 504,
                    error_type: "stream_idle_timeout",
                },
            ),
            (
                "upstream_response_read_error",
                StreamCooldownFailure {
                    status_code: 502,
                    error_type: "upstream_response_read_error",
                },
            ),
        ];

        for (error_type, expected) in cases {
            let summary = StreamTerminalSummary::provider_failure(error_type, "provider failed", &StreamStatus::default());

            assert_eq!(summary.record_status, "failed");
            assert_eq!(summary.cooldown, Some(expected));
        }
    }

    #[test]
    fn conversion_failure_does_not_enter_provider_cooldown() {
        let summary = StreamTerminalSummary::provider_failure("response_conversion_error", "bad conversion", &StreamStatus::default());

        assert_eq!(summary.record_status, "failed");
        assert_eq!(summary.status_code, 502);
        assert_eq!(summary.cooldown, None);
    }

    #[test]
    fn successful_terminal_summary_has_no_client_failure_or_cooldown() {
        let summary = StreamTerminalSummary::success(axum::http::StatusCode::OK, &StreamStatus::default());

        assert_eq!(summary.record_status, "success");
        assert_eq!(summary.client_failure(), None);
        assert_eq!(summary.cooldown, None);
    }
}
