# Progress

## 2026-06-01

- Inspected the existing flow.
- Frontend `ProviderModelDialog.saveChanges()` loops through removals and additions.
- Backend exposes only single-binding create and delete operations.
- Planned a collection-level batch update with pre-validation and transactional storage.
- Added `ProviderModelBindingBatchUpdate` and `/api/admin/providers/{provider_id}/models/batch-update`.
- Storage applies deletes and creates inside one SeaORM transaction and returns the final provider binding list.
- Frontend provider model dialog now sends one batch request for all pending add/remove changes.
- Validation passed for focused storage/provider/backend tests, frontend lint/build, formatting, and related clippy.
- Full `just test` still fails in unrelated `crates/formats` tests that compare JSON field order.
