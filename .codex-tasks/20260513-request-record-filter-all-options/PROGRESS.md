# Progress

## 2026-05-13

- Read the request records page, request records toolbar, and the working provider/admin toolbar implementations.
- Confirmed the request records status/format/type selects are the only ones using an empty string sentinel for the all-option value.
- Confirmed request record API filters already treat missing values as optional, so the frontend fix should stay local to filter state plus serialization.
- Updated the request records toolbar to use the same explicit `all` sentinel pattern as the working provider/admin filters.
- Updated request record query serialization so both `all` and blank values map back to omitted API filters.
- Validation passed: `pnpm --filter hook_frontend lint`, `pnpm --filter hook_frontend build`, and `git diff --check`.
