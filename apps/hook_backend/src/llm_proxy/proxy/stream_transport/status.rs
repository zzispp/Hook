use time::OffsetDateTime;
use types::model::PatchField;

const MAX_STREAM_ERROR_ENTRIES: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StreamEndReason {
    Done,
    Timeout,
    ClientGone,
    ScannerError,
    HandlerStop,
    Eof,
    UpstreamEofWithoutCompletion,
}

impl StreamEndReason {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Timeout => "timeout",
            Self::ClientGone => "client_gone",
            Self::ScannerError => "scanner_error",
            Self::HandlerStop => "handler_stop",
            Self::Eof => "eof",
            Self::UpstreamEofWithoutCompletion => "upstream_eof_without_completion",
        }
    }

    pub(super) fn is_normal(self) -> bool {
        matches!(self, Self::Done | Self::Eof | Self::HandlerStop)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StreamErrorEntry {
    pub(super) message: String,
    pub(super) recorded_at: OffsetDateTime,
}

#[derive(Default)]
pub(super) struct StreamStatus {
    end_reason: Option<StreamEndReason>,
    end_error: Option<String>,
    errors: Vec<StreamErrorEntry>,
    error_count: usize,
    received_response_count: usize,
}

impl StreamStatus {
    pub(super) fn set_end_reason(&mut self, reason: StreamEndReason, error: Option<String>) {
        if self.end_reason.is_some() {
            return;
        }
        self.end_reason = Some(reason);
        self.end_error = error;
    }

    pub(super) fn record_error(&mut self, message: impl Into<String>) {
        self.error_count += 1;
        if self.errors.len() >= MAX_STREAM_ERROR_ENTRIES {
            return;
        }
        self.errors.push(StreamErrorEntry {
            message: message.into(),
            recorded_at: OffsetDateTime::now_utc(),
        });
    }

    pub(super) fn record_response(&mut self) {
        self.received_response_count += 1;
    }

    pub(super) fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub(super) fn total_error_count(&self) -> usize {
        self.error_count
    }

    pub(super) fn received_response_count(&self) -> usize {
        self.received_response_count
    }

    pub(super) fn is_normal_end(&self) -> bool {
        match self.end_reason {
            Some(reason) => reason.is_normal(),
            None => true,
        }
    }

    pub(super) fn end_reason(&self) -> Option<StreamEndReason> {
        self.end_reason
    }

    pub(super) fn stream_end_reason_patch(&self) -> PatchField<String> {
        match self.end_reason {
            Some(reason) => PatchField::Value(reason.as_str().into()),
            None => PatchField::Null,
        }
    }

    pub(super) fn summary(&self) -> String {
        let reason = self.end_reason.map(StreamEndReason::as_str).unwrap_or("");
        let mut summary = format!("reason={reason}");
        if let Some(error) = &self.end_error {
            summary.push_str(&format!(" end_error={error:?}"));
        }
        if self.error_count > 0 {
            summary.push_str(&format!(" soft_errors={}", self.error_count));
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use types::model::PatchField;

    use super::{StreamEndReason, StreamStatus};

    #[test]
    fn set_end_reason_keeps_first_reason() {
        let mut status = StreamStatus::default();

        status.set_end_reason(StreamEndReason::Done, None);
        status.set_end_reason(StreamEndReason::Timeout, Some("timeout".into()));

        assert_eq!(status.stream_end_reason_patch(), PatchField::Value("done".into()));
        assert_eq!(status.summary(), "reason=done");
    }

    #[test]
    fn normal_end_matches_stream_semantics() {
        for reason in [StreamEndReason::Done, StreamEndReason::Eof, StreamEndReason::HandlerStop] {
            let mut status = StreamStatus::default();
            status.set_end_reason(reason, None);

            assert!(status.is_normal_end(), "{reason:?} should be normal");
        }
    }

    #[test]
    fn abnormal_end_matches_stream_semantics() {
        for reason in [
            StreamEndReason::Timeout,
            StreamEndReason::ClientGone,
            StreamEndReason::ScannerError,
            StreamEndReason::UpstreamEofWithoutCompletion,
        ] {
            let mut status = StreamStatus::default();
            status.set_end_reason(reason, None);

            assert!(!status.is_normal_end(), "{reason:?} should be abnormal");
        }
    }

    #[test]
    fn records_soft_error_count_and_caps_entries() {
        let mut status = StreamStatus::default();

        for index in 0..25 {
            status.record_error(format!("error_{index}"));
        }

        assert!(status.has_errors());
        assert_eq!(status.total_error_count(), 25);
        assert_eq!(status.errors.len(), 20);
    }

    #[test]
    fn counts_received_response_events() {
        let mut status = StreamStatus::default();

        status.record_response();
        status.record_response();

        assert_eq!(status.received_response_count(), 2);
    }

    #[test]
    fn maps_all_end_reasons_to_client_style_strings() {
        let cases = [
            (StreamEndReason::Done, "done"),
            (StreamEndReason::Timeout, "timeout"),
            (StreamEndReason::ClientGone, "client_gone"),
            (StreamEndReason::ScannerError, "scanner_error"),
            (StreamEndReason::HandlerStop, "handler_stop"),
            (StreamEndReason::Eof, "eof"),
            (StreamEndReason::UpstreamEofWithoutCompletion, "upstream_eof_without_completion"),
        ];

        for (reason, expected) in cases {
            assert_eq!(reason.as_str(), expected);
        }
    }

    #[test]
    fn terminal_patch_keeps_done_eof_and_incomplete_reasons_visible() {
        for (reason, expected) in [
            (StreamEndReason::Done, "done"),
            (StreamEndReason::Eof, "eof"),
            (StreamEndReason::UpstreamEofWithoutCompletion, "upstream_eof_without_completion"),
        ] {
            let mut status = StreamStatus::default();
            status.set_end_reason(reason, None);

            assert_eq!(status.stream_end_reason_patch(), PatchField::Value(expected.into()));
        }
    }
}
