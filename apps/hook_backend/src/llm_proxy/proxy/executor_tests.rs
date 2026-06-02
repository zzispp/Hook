use std::time::Duration;

use super::{AttemptOnceOutcome, StreamWatchdogOutcome, probe_slot_timeout_outcome, run_stream_candidate_watchdog, stream_candidate_watchdog_timeout_output};
use crate::llm_proxy::{LlmProxyError, proxy::attempt_log::AttemptCancelHandle};

#[tokio::test]
async fn stream_candidate_watchdog_returns_completed_result_before_timeout() {
    let outcome = run_stream_candidate_watchdog(Some(Duration::from_millis(50)), AttemptCancelHandle::noop_for_test(), async {
        Ok::<_, LlmProxyError>(7_i32)
    })
    .await
    .expect("watchdog should finish");

    assert!(matches!(outcome, StreamWatchdogOutcome::Completed(7)));
}

#[tokio::test]
async fn stream_candidate_watchdog_times_out_pending_task() {
    let outcome = run_stream_candidate_watchdog(Some(Duration::from_millis(5)), AttemptCancelHandle::noop_for_test(), async {
        std::future::pending::<Result<i32, LlmProxyError>>().await
    })
    .await
    .expect("watchdog timeout should be handled");

    assert!(matches!(outcome, StreamWatchdogOutcome::TimedOut));
}

#[test]
fn stream_candidate_watchdog_timeout_advances_to_next_candidate() {
    let output = stream_candidate_watchdog_timeout_output();

    assert!(matches!(output.outcome, AttemptOnceOutcome::NextCandidate));
    assert!(output.last_failure.is_some());
    assert_eq!(
        output.last_error.map(|error| error.to_string()),
        Some("stream candidate watchdog timed out".into())
    );
}

#[test]
fn probe_slot_timeout_continues_candidate_route() {
    let output = probe_slot_timeout_outcome();

    assert!(matches!(output, AttemptOnceOutcome::ContinueCandidate));
}
