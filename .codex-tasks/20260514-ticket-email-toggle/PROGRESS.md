# Progress

## 2026-05-14

- Started Full Single task tracking for the ticket email notification toggle.
- Added persisted `support_ticket_email_notifications_enabled` system setting across types, storage entity, repository patch, and baseline seed.
- Updated setting validation so ticket email notifications require enabled email configuration plus complete SMTP settings, matching registration email verification behavior.
- Changed ticket mailer delivery behavior: disabled ticket notifications return `disabled` without an error; enabled notifications still surface configuration and delivery failures explicitly.
- Added frontend settings switch and localized ticket email failure messages via backend admin i18n seed keys.
- Validation passed: `cargo fmt --all`, `cargo check --workspace`, `pnpm lint:frontend`, `pnpm build:frontend`, and 60-second-wrapped `cargo test --workspace`.
