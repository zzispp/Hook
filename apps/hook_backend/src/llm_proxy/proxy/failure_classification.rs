use req::StatusCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::llm_proxy) enum FailureDecision {
    ReturnResponse,
    NextCandidate,
    RetryOrNextCandidate,
}

impl FailureDecision {
    pub(super) fn records_provider_cooldown(self) -> bool {
        matches!(self, Self::RetryOrNextCandidate)
    }
}

pub(in crate::llm_proxy) fn classify_status(status: StatusCode) -> FailureDecision {
    if matches!(
        status,
        StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::PAYMENT_REQUIRED | StatusCode::FORBIDDEN
    ) {
        return FailureDecision::NextCandidate;
    }
    if status.is_server_error() || matches!(status, StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS) {
        return FailureDecision::RetryOrNextCandidate;
    }
    FailureDecision::ReturnResponse
}

#[cfg(test)]
mod tests {
    use req::StatusCode;

    use super::{FailureDecision, classify_status};

    #[test]
    fn auth_failures_switch_candidate_without_cooldown_recording() {
        assert_eq!(classify_status(StatusCode::BAD_REQUEST), FailureDecision::NextCandidate);
        assert_eq!(classify_status(StatusCode::UNAUTHORIZED), FailureDecision::NextCandidate);
        assert_eq!(classify_status(StatusCode::PAYMENT_REQUIRED), FailureDecision::NextCandidate);
        assert_eq!(classify_status(StatusCode::FORBIDDEN), FailureDecision::NextCandidate);
        assert!(!classify_status(StatusCode::BAD_REQUEST).records_provider_cooldown());
        assert!(!classify_status(StatusCode::FORBIDDEN).records_provider_cooldown());
    }

    #[test]
    fn rate_timeout_and_server_failures_are_retryable_and_cooldown_eligible() {
        assert_eq!(classify_status(StatusCode::TOO_MANY_REQUESTS), FailureDecision::RetryOrNextCandidate);
        assert_eq!(classify_status(StatusCode::REQUEST_TIMEOUT), FailureDecision::RetryOrNextCandidate);
        assert!(classify_status(StatusCode::BAD_GATEWAY).records_provider_cooldown());
    }

    #[test]
    fn other_client_request_errors_return_directly() {
        assert_eq!(classify_status(StatusCode::NOT_FOUND), FailureDecision::ReturnResponse);
    }
}
