# Password UI Template Migration Progress

## 2026-06-01

- Started from the accepted plan.
- Verified current forgot-password and profile password implementations.
- Migrated forgot-password UI to the split reset-password visual structure.
- Replaced the profile password card with a hook-form based split update-password style form.
- Fixed password reset auth actions to reject `success:false` API envelopes so backend errors are shown instead of a false success state.
- Validation passed: `pnpm lint:frontend`, `pnpm build:frontend`.
- User clarified that profile password change must move to a standalone page using the split update-password template UI.
- Resuming with standalone route, profile jump entry, and dashboard_profile account API seed permissions.
- Added `/dashboard/profile/change-password` with the split update-password structure and kept `changeAccountPassword` / `requestAccountPasswordEmailCode` as the only business calls.
- Changed `/dashboard/profile` to show local password status plus a jump button instead of the inline form.
- Added account self-service API definitions and bound them to `dashboard_profile`; removed account self-service paths from `auth.authenticated` so seed permissions participate in authorization.
- Validation passed again: `pnpm lint:frontend`, `pnpm build:frontend`, `just check`.
- User asked whether unverified email should have its own verification action and whether successful email-code operations should mark email verified.
- Decision: add a standalone verify-email page and mark `email_verified` true after any successful current-email code consumption, starting with account password change and verify-email.
- Added backend `verify_account_email` flow using the existing account email code purpose; password change now sets `email_verified` after consuming the current-email code.
- Added `/dashboard/profile/verify-email` and reused the same profile email-code fields shared by the password page.
- Validation passed: `cargo test -p user account_`, `pnpm lint:frontend`, `pnpm build:frontend`, `just check`.
- User asked profile verify-email/password-change to respect the admin email config switch.
- Added account profile `email_code_available` from backend mail readiness and kept account email-code sending tied to account mail settings, not registration-email verification.
- Password change now uses email code when mail is ready; when mail is unavailable it requires current password, and passwordless users receive an explicit admin-reset error.
- Frontend profile buttons now toast before navigation when the requested operation cannot work; change-password page switches between email-code and current-password forms.
- Validation: `cargo test -p user account_password_change -- --nocapture`, `pnpm build:frontend`, `just check` passed. `pnpm lint:frontend` remains blocked by existing admin provider endpoint lint errors outside this change.
