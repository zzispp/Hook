use super::StreamRelay;
use crate::llm_proxy::proxy::stream_transport::{
    record::{
        StreamAttemptRecord, StreamCancelledReason, StreamCancelledRecordInput, StreamTerminalRecordInput, cancelled_record, terminal_stream_record,
    },
    status::{StreamEndReason, StreamStatus},
    terminal::StreamTerminalSummary,
};
use crate::llm_proxy::proxy::attempt_log::{AttemptCancelReason, take_candidate_cancel_reason};

pub(super) fn drop_terminal_record(relay: &mut StreamRelay) -> StreamAttemptRecord {
    match drop_record_kind(relay.protocol_completed, &relay.stream_status) {
        StreamDropRecordKind::Completed => completed_drop_record(relay),
        StreamDropRecordKind::Cancelled => cancelled_drop_record(relay),
    }
}

fn completed_drop_record(relay: &mut StreamRelay) -> StreamAttemptRecord {
    relay.stream_status.set_end_reason(StreamEndReason::Done, None);
    let summary = StreamTerminalSummary::success(relay.context.status, &relay.stream_status).with_observability(relay.terminal_observability());
    terminal_stream_record(StreamTerminalRecordInput {
        context: &relay.context,
        usage: relay.usage,
        summary,
    })
}

fn cancelled_drop_record(relay: &mut StreamRelay) -> StreamAttemptRecord {
    let reason = stream_cancel_reason(relay);
    let (end_reason, message, record_reason) = match reason {
        AttemptCancelReason::ClientDisconnect => (
            StreamEndReason::ClientGone,
            "client disconnected before stream completed",
            StreamCancelledReason::ClientDisconnected,
        ),
        AttemptCancelReason::HedgedBackupSuperseded => (
            StreamEndReason::HedgeCancelled,
            "stream attempt cancelled because backup stream won",
            StreamCancelledReason::HedgedBackupSuperseded,
        ),
    };
    relay.stream_status.set_end_reason(end_reason, Some(message.into()));
    cancelled_record(StreamCancelledRecordInput {
        context: &relay.context,
        usage: relay.usage,
        status: &relay.stream_status,
        observability: relay.cancelled_observability(),
        reason: record_reason,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StreamDropRecordKind {
    Completed,
    Cancelled,
}

fn drop_record_kind(protocol_completed: bool, status: &StreamStatus) -> StreamDropRecordKind {
    match status.end_reason() {
        Some(StreamEndReason::Done) => StreamDropRecordKind::Completed,
        None if protocol_completed => StreamDropRecordKind::Completed,
        _ => StreamDropRecordKind::Cancelled,
    }
}

fn stream_cancel_reason(relay: &StreamRelay) -> AttemptCancelReason {
    take_candidate_cancel_reason(&relay.context.request_id, relay.context.candidate.trace.candidate_index)
        .unwrap_or_else(|| relay.context.cancel_handle.reason())
}

#[cfg(test)]
mod tests {
    use super::{StreamDropRecordKind, drop_record_kind};
    use crate::llm_proxy::proxy::stream_transport::status::{StreamEndReason, StreamStatus};

    #[test]
    fn drop_records_completed_when_stream_already_reached_done() {
        let mut status = StreamStatus::default();
        status.set_end_reason(StreamEndReason::Done, None);

        assert_eq!(drop_record_kind(false, &status), StreamDropRecordKind::Completed);
    }

    #[test]
    fn drop_records_completed_when_protocol_completed_before_done_patch() {
        let status = StreamStatus::default();

        assert_eq!(drop_record_kind(true, &status), StreamDropRecordKind::Completed);
    }

    #[test]
    fn drop_records_cancelled_before_protocol_completion() {
        let status = StreamStatus::default();

        assert_eq!(drop_record_kind(false, &status), StreamDropRecordKind::Cancelled);
    }

    #[test]
    fn drop_preserves_failure_classification_after_stream_error() {
        let mut status = StreamStatus::default();
        status.set_end_reason(StreamEndReason::ScannerError, Some("decode failed".into()));

        assert_eq!(drop_record_kind(true, &status), StreamDropRecordKind::Cancelled);
    }
}
