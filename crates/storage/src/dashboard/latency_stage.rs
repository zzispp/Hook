#[derive(Clone, Copy, Debug, Default)]
pub(super) struct StageLatencyContribution {
    pub(super) response_headers_ms: Option<i64>,
    pub(super) first_sse_event_ms: Option<i64>,
    pub(super) first_output_ms: Option<i64>,
    pub(super) sse_to_output_ms: Option<i64>,
}

impl StageLatencyContribution {
    pub(super) fn new(response_headers_ms: Option<i64>, first_sse_event_ms: Option<i64>, first_output_ms: Option<i64>) -> Self {
        let response_headers_ms = non_negative(response_headers_ms);
        let first_sse_event_ms = non_negative(first_sse_event_ms);
        let first_output_ms = non_negative(first_output_ms);
        Self {
            response_headers_ms,
            first_sse_event_ms,
            first_output_ms,
            sse_to_output_ms: sse_to_output(first_sse_event_ms, first_output_ms),
        }
    }

    pub(super) fn total(value: Option<i64>) -> i64 {
        value.unwrap_or_default()
    }

    pub(super) fn sample_count(value: Option<i64>) -> i64 {
        i64::from(value.is_some())
    }
}

fn non_negative(value: Option<i64>) -> Option<i64> {
    value.filter(|item| *item >= 0)
}

fn sse_to_output(first_sse_event_ms: Option<i64>, first_output_ms: Option<i64>) -> Option<i64> {
    first_output_ms?.checked_sub(first_sse_event_ms?).filter(|value| *value >= 0)
}
