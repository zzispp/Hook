# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Replace pure routing rankings/preview with token-aware routing dry-run.
- Make admin routing simulation use the same token, group, model access, cache affinity, scheduler seed, and effective profile path as real requests.
- Keep dry-run read-only: no upstream request, no request records, no decision sample writes, no routing metric/EMA/affinity/circuit writes.

## Non-Goals

- No database migration.
- No compatibility UI for the previous no-token pure strategy rankings.
- No profile override in simulation.

## Constraints

- `/api/admin/routing/rankings` remains the main admin rankings endpoint, but its query becomes token-aware.
- `/api/admin/routing/preview` is removed because it only represents the old pure preview.
- `request_id_seed` is optional; when omitted, the backend generates a UUID and returns it.
- Frontend keeps a stable seed for the current simulation context to avoid auto-refresh seed jitter.

## Deliverables

- Backend token-aware rankings request/response types and handler.
- Shared read-only routing selection helper used by both real request selection and dry-run.
- Removed preview route/types/helpers/default API seed.
- Frontend routing observability page uses active token selection and response profile.
- Tests covering cache affinity, token validation, profile derivation, seed stability, and UI type/lint safety.

## Done-When

- Token-aware rankings returns the same affinity-influenced candidate ordering that a real request would use.
- Rankings cannot be requested without a valid active unexpired token.
- Profile is derived from model/group defaults and cannot be overridden by query parameters.
- Old pure preview UI/API path is gone.
- Validation commands complete or any timeout/failure is reported with exact output.

