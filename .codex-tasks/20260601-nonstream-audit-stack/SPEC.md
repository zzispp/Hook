# Nonstream Audit Stack Refactor

## Goal

Remove the non-stream full response tokio::spawn workaround and reduce stack pressure by restructuring the full_response -> record_attempt -> billing/storage path.

## Acceptance Criteria

- Non-stream success response does not use tokio::spawn as a stack workaround.
- Audit persistence uses a lighter owned input across await boundaries where practical.
- Existing billing semantics and request/candidate persistence behavior remain unchanged.
- cargo fmt --all and cargo check pass.
- Focused storage tests pass.
- The reproduced /v1/chat/completions curl returns 200 without stack overflow.
