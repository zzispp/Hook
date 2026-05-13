# Admin Email Settings

## Goal

Add email configuration to the admin system settings page, following existing Hook project conventions and referencing Aether's email feature where useful.

## Scope

- Add SMTP configuration fields to the backend settings model, persistence defaults, admin API contract, and frontend system settings form.
- Add a registration email verification switch near the existing login and registration captcha settings.
- Add configurable email templates for registration verification code and password reset, with visible allowed variables.
- Update backend admin i18n seed resources instead of frontend locale JSON files.
- Validate with focused backend and frontend checks.

## Non-goals

- Do not implement real email sending unless the existing Hook code path already contains a mail sender to wire.
- Do not add compatibility fallbacks, mock success paths, or silent degradation.

