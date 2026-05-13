# Email Config Enable Switch

## Goal

Add an explicit switch in admin system settings that controls whether saved email configuration is active.

## Scope

- Add persisted setting support for enabling email configuration.
- Update admin system settings UI so email configuration can be saved while disabled.
- Ensure email verification can only be enabled when email configuration is enabled and complete.
- Validate with focused automated checks where available.

## Constraints

- Do not add silent fallbacks or mock success behavior.
- Preserve backend-controlled admin i18n behavior.
- Keep changes scoped to email system settings and verification gating.
