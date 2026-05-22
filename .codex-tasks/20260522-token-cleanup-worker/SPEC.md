# Token Cleanup Worker

## Goal

- Move expired API token cleanup from token list request handlers into a standalone scheduled worker.
- Reuse the existing system setting `token_expiry_check_interval_minutes` as the worker interval.
- Keep `auto_delete_expired_tokens` as the runtime enable switch.

## Scope

- Existing backend worker patterns.
- API token service and API handlers.
- Backend startup wiring.
- Focused backend tests and validation.
