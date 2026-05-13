# Progress

## Recovery

- Task: Add admin email configuration enable switch and gate email verification.
- Shape: single-full
- Current truth: `.codex-tasks/20260513-email-config-switch/TODO.csv`
- Current step: 1

## Log

- 2026-05-13: Created task record and started codebase inspection.
- 2026-05-13: Located system settings types, storage entity/repository, setting service, admin UI form, and backend i18n seed files. Registration email verification currently appears only as a setting; no email sending execution path was found in the inspected backend scope.
- 2026-05-13: Added `email_config_enabled` to backend system setting types, storage, baseline schema/seed, and service-level email verification prerequisite validation. `cargo test -p setting` passed under the 60-second wrapper.
- 2026-05-13: Added the admin email configuration switch, disabled registration email verification enablement until prerequisites are ready, and updated backend-controlled admin i18n seeds. `pnpm lint:frontend` passed after an import-order fix.
