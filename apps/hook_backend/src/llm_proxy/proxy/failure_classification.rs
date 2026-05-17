use req::StatusCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FailureDecision {
    ReturnResponse,
    NextCandidate,
    RetryOrNextCandidate,
}

impl FailureDecision {
    pub(super) fn records_provider_cooldown(self) -> bool {
        matches!(self, Self::RetryOrNextCandidate)
    }
}

pub(super) fn classify_status(status: StatusCode) -> FailureDecision {
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
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
        assert_eq!(classify_status(StatusCode::UNAUTHORIZED), FailureDecision::NextCandidate);
        assert!(!classify_status(StatusCode::FORBIDDEN).records_provider_cooldown());
    }

    #[test]
    fn rate_timeout_and_server_failures_are_retryable_and_cooldown_eligible() {
        assert_eq!(classify_status(StatusCode::TOO_MANY_REQUESTS), FailureDecision::RetryOrNextCandidate);
        assert_eq!(classify_status(StatusCode::REQUEST_TIMEOUT), FailureDecision::RetryOrNextCandidate);
        assert!(classify_status(StatusCode::BAD_GATEWAY).records_provider_cooldown());
    }

    #[test]
    fn client_request_errors_return_directly() {
        assert_eq!(classify_status(StatusCode::BAD_REQUEST), FailureDecision::ReturnResponse);
        assert_eq!(classify_status(StatusCode::NOT_FOUND), FailureDecision::ReturnResponse);
    }
}
