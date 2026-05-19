# Token Expiry Future Validation

## Goal

Prevent API token creation and update from accepting an expires_at value that is already in the past.

## Scope

- Backend API token validation in crates/api_token.
- Frontend API token datetime controls if needed after backend behavior is verified.
- Tests that prove past expires_at is rejected.
