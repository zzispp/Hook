# Provider Model Batch Bindings

## Goal

Replace the provider model association dialog's per-binding HTTP loop with one batch request.

## Scope

- Add a collection-level backend API for provider model binding changes.
- Validate all new bindings before persistence.
- Apply creates and deletes atomically in storage.
- Update the frontend dialog to submit one request for all pending changes.

## Acceptance Criteria

- Selecting or clearing multiple provider models sends one HTTP request.
- A failed batch does not persist partial association changes.
- Existing single-binding create, update, delete, and test APIs keep working.
- Backend tests, Rust formatting, and frontend lint/build checks pass.
