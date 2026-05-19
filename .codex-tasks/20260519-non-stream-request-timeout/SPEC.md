# Non-stream request timeout

## Goal

Use provider `request_timeout_seconds` as the total timeout budget for non-stream proxy requests.

## Scope

- Do not touch production DB or historical records.
- Keep streaming governed by `stream_first_byte_timeout_seconds` and `stream_idle_timeout_seconds`.
- Make timeout failures visible as `upstream_timeout`.

## Evidence

- Hook currently stores and propagates `request_timeout_seconds`.
- Hook currently sets reqwest request timeout for non-stream requests in outbound request building.
- Aether wraps upstream request execution in `tokio::time::timeout(timeout, client.request(request))`.

