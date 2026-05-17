# Sanitize LLM Upstream Errors

## Goal

Prevent LLM proxy client responses from exposing upstream provider bodies, hostnames, URLs, keys, or internal infrastructure details when every candidate fails or a single upstream error is returned.

## Boundary

- Client-facing LLM proxy errors must use fixed sanitized messages and codes.
- Audit records must keep the original upstream response body, headers, and internal error text for debugging.
- Existing candidate switching, cooldown, and failure classification behavior must remain intact.
- No mock success path or silent fallback is allowed.

## Expected Behavior

- Upstream HTTP failures returned to the client use a Hook-owned JSON error body.
- `LlmProxyError::Upstream` and `LlmProxyError::Infrastructure` do not expose their internal string in HTTP responses.
- `Display` and audit paths can still use the original internal strings.

## Validation

- Unit tests prove upstream failure bodies are not returned to clients.
- Unit tests prove `LlmProxyError::Upstream` and `Infrastructure` responses are sanitized.
- Rust formatting and backend checks pass under a 60 second timeout.
