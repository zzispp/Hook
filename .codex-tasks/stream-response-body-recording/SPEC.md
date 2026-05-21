# Stream Response Body Recording

## Goal

Persist Provider and client response body payloads for successful, failed, and cancelled streaming proxy attempts when the existing request-record response body switches and record levels allow it.

## Scope

- Backend streaming proxy recording only.
- Reuse existing request-record policies and truncation behavior.
- Preserve current stream status, usage, timeout, cancellation, and error behavior.

## Out Of Scope

- Frontend UI redesign.
- New storage columns.
- New fallback behavior or synthetic success paths.
