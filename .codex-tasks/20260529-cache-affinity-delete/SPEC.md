# Fix Cache Affinity Delete Response Handling

## Goal

Deleting a cache affinity from the admin frontend must not show `Response data not found` when the backend delete operation succeeds.

## Scope

- Inspect the frontend cache monitoring delete action.
- Confirm the backend response shape for cache affinity deletion.
- Apply the smallest contract-correct frontend fix.
- Run feasible validation for the changed frontend code.

## Non-Goals

- Do not change backend response envelopes globally.
- Do not add fallback or mock success paths.
