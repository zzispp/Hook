# Operations Modules

## Goal

Add production-backed announcement, support ticket, and notification modules under Operations Management.

## Scope

- Backend Rust workspace: operations domain crate, storage, routes, migration tables, default APIs, menu bindings, and email delivery through existing SMTP settings.
- Frontend dashboard: user/admin announcement pages, user/admin ticket pages, notification drawer backed by APIs, routes, menu metadata, and admin i18n seed updates.
- Validation: run Rust checks and frontend lint/build checks where feasible.

## Non-Goals

- Attachments for support tickets.
- Mock or fallback success paths.
- Frontend admin locale JSON files.

## Acceptance

- Admin can manage announcements and tickets.
- Users can view announcements and manage their own tickets.
- Ticket submission defaults to the account email and allows explicit editing.
- Admin notification drawer lists user-submitted/updated tickets; user notification drawer lists announcements and ticket updates.
- Notifications support read/unread and per-user deletion.
- Email delivery uses configured SMTP and records visible failures.
