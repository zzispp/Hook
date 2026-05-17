# Progress

## 2026-05-17

- Confirmed current leak paths:
  - `proxy::executor` returns `transport::failure_response` after all candidates fail.
  - `transport::failure_response` returns the original upstream body.
  - `LlmProxyError::Upstream` and `Infrastructure` currently serialize their internal message into HTTP responses.
- Added red tests for:
  - Upstream HTTP failure response body sanitization.
  - Upstream/internal `LlmProxyError` HTTP response sanitization while preserving `Display`.
- Implemented sanitized client responses:
  - Shared client JSON error definitions live in `apps/hook_backend/src/llm_proxy/client_error.rs`.
  - Non-stream upstream failures record original provider body/headers in audit but write sanitized client response body.
  - Stream pre-response upstream failures use the same sanitized client body.
  - `LlmProxyError::Upstream` and `Infrastructure` keep internal `Display` text but expose fixed client messages/codes through `IntoResponse`.
- Validation passed:
  - `cargo fmt`
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p backend llm_proxy:: -- --nocapture`
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo check --workspace`
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo test --workspace`
- Added real script `real_sanitize_failure_flow.mjs`:
  - Seeds one model/token/provider against a local upstream that returns a 429 body containing sensitive provider markers.
  - Calls Hook `/v1/chat/completions`.
  - Asserts client response and client audit body are sanitized.
  - Asserts provider audit body and candidate error message retain the raw upstream details.
- Real script validation passed:
  - Command: `perl -e 'alarm shift; exec @ARGV' 120 node .codex-tasks/20260517-sanitize-llm-upstream-errors/real_sanitize_failure_flow.mjs`
  - Evidence written to `raw/sanitize-results.json`.
  - Client HTTP response status is `502`, body code is `model_service_unavailable`, and no sensitive upstream marker is present.
  - `request_records.client_response_body` is sanitized.
  - Provider audit fields retain the upstream marker for internal debugging.
