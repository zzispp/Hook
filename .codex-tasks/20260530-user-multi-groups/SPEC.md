# User Multi Groups

## Objective

Implement user-to-user-group many-to-many membership. User API payloads and responses expose `group_codes` only. Billing group `group_code` on API tokens remains billing-group selection.

## Scope

- Baseline schema only; no existing database data migration path.
- Registration default group policy remains backed by `system_settings.default_user_group_code`.
- Users must have at least one active user group.
- User list filters may keep a singular `group_code` predicate.

## Validation

- Rust tests cover membership storage, token visibility, available groups, and proxy model access.
- Frontend lint/build validates TypeScript and UI wiring.

