# Ticket Email Notification Toggle

## Goal

Add an explicit system setting switch for support ticket email notifications and localize visible delivery messages.

## Scope

- Backend system settings schema/types/storage/API payloads.
- Settings validation: enabling ticket email notifications requires enabled and complete email configuration.
- Operations mailer: skip ticket email delivery when the ticket notification switch is disabled.
- Frontend settings form and operations ticket toast behavior.
- Admin i18n seed updates.

## Acceptance

- Ticket replies do not show an email configuration error when ticket email notifications are disabled.
- Enabling ticket email notifications is rejected unless SMTP email configuration is enabled and complete.
- Delivery errors displayed in the UI use backend/admin i18n keys instead of raw English infrastructure strings.
- Existing frontend/backend checks pass.
