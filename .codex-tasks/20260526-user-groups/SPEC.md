# User Groups Billing Visibility

## Goal

Implement user groups as a single business segment per user, bind billing groups to user groups for visibility, and expose the required admin UI.

## Decisions

- User belongs to exactly one user group through `users.group_code`.
- `default` user group is system-owned, always enabled, not deletable, not disableable, and not renameable by code.
- Billing group empty visibility bindings mean no normal user can see it.
- Billing group creation UI selects `default` by default.
- System settings include `default_user_group_code`, defaulting to `default`.
- Existing development baseline can be changed directly; no compatibility migration is required.

## Validation

- Backend tests for default group rules, registration assignment, billing group visibility, settings validation, and token policy.
- Frontend lint/build for admin UI.
- `cargo fmt --all`, `just test`, and relevant clippy checks.
