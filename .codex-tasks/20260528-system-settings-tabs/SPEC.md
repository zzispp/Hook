# System Settings Tabs

## Goal

Change the admin system settings page from stacked sections into page-level tabs matching the recharge management page pattern.

## Scope

- Add tabs for site, registration, email, tokens, recharge, and request record settings.
- Keep cleanup task configuration in scheduled task management.
- Keep the existing system settings API and field model unchanged.
- Update backend admin i18n seed keys required by the new tab labels.

## Validation

- `pnpm lint:frontend`
- `pnpm build:frontend`

