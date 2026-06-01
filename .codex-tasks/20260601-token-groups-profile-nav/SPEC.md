# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Admin token creation can select any active billing group, independent of the token owner's user groups.
- Profile remains reachable from the account drawer and route, but is removed from default dashboard role menus for admin and user roles.

## Non-Goals

- Do not delete the profile page, route, account drawer entry, or profile translations.
- Do not change normal user token group visibility rules.

## Constraints

- Keep backend failures explicit.
- Preserve existing RBAC/menu seed structure.
- Keep frontend changes scoped to token group selection and stale prop removal.

## Deliverables

- Backend admin token create policy split from normal owner visibility checks.
- Frontend admin token group selector lists all active billing groups.
- Default admin/user role menu seed excludes `dashboard_profile`.
- Focused tests and lint validation.

## Done-When

- `cargo test -p api_token admin` passes.
- `cargo test -p backend defaults` passes.
- `pnpm lint:frontend` passes.
- `cargo fmt -p api_token --check && cargo fmt -p backend --check` passes.
