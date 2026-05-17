# Progress

## 2026-05-18

- Started from user report: record 019e36d7 appears to correspond to CLI streaming where Aether produces multiple request records but Hook does not.

- DB evidence: openai_cli stream records for 019e36d7 and later requests were created as multiple records, but request_records/request_candidates stayed at status=streaming, billing_status=pending, finished_at=NULL.
- Root cause found in Hook stream path: StreamRelay only finalized success on upstream EOF; Aether finalizes on protocol completion/cancelled telemetry. CLI clients can close after response.completed, leaving Hook waiting for EOF. Drop used best-effort tokio::spawn and could not be the primary audit boundary.
- Implemented parser-level completion detection for OpenAI Responses response.completed, OpenAI [DONE], Claude message_stop, and Gemini finishReason; Relay now finalizes success after protocol completion once first-byte audit is written.

- Validation: targeted cargo test for stream_transport::usage_parser passed. cargo check passed. Real CLI stream script against temporary backend :5566 passed with request 019e36ee-cd20-73d3-8085-eb5e404a7130 => request_records success/settled, candidate success, tokens 30/11/41, cost 0.00048000.
