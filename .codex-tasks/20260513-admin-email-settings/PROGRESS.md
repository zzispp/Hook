# Progress

## Recovery

- 任务: 在后台系统设置加入邮件配置与模板
- 形态: single-full
- 进度: 12/12
- 当前: Completed
- 文件: `.codex-tasks/20260513-admin-email-settings/TODO.csv`
- 下一步: 无。

## Log

- 2026-05-13: Started task and created tracking artifacts.
- 2026-05-13: Confirmed Hook settings path is `system_settings` storage -> `types::system_setting` -> `setting` API -> admin form. Aether reference fields are SMTP host/port/user/password/from name/from email, TLS/SSL flags, email suffix mode/list, and templates for verification/password reset with variables.
- 2026-05-13: Extended backend settings contract, storage patching, baseline seed/table definitions, and encrypted SMTP password handling. `cargo test -p setting` passed.
- 2026-05-13: Added frontend email settings section, registration email verification switch, form mapping, and admin i18n keys. `pnpm lint:frontend` passed.
- 2026-05-13: Backend package tests passed after i18n seed updates.
- 2026-05-13: Final validation passed: `cargo fmt --check --all`, full `cargo test` with a 60-second timeout, `pnpm lint:frontend`, and `pnpm build:frontend`. `pnpm build:frontend` still prints the existing `Axios error: unauthorized` during static generation but exits successfully.
- 2026-05-13: Browser smoke reached `/dashboard/admin/settings` and rendered the new email section. The running local database still has old admin i18n entries, so new email labels appear as raw keys in that browser session until the seed data is reapplied.
- 2026-05-13: Split baseline migration identifiers into domain modules so touched migration files stay under the project file-size rule. Re-ran `cargo fmt --check --all`, `cargo test -p backend`, and full `cargo test` with a 60-second timeout; all passed.
- 2026-05-13: Reopened task for template polish. Confirmed Hook palette uses Minimal UI primary green (`#00A76F`, `#007867`, `#004B50`) with grey surfaces/borders; default template HTML exists in backend setting seed and frontend form defaults.
- 2026-05-13: Updated registration and password-reset HTML defaults in backend seed and frontend fallback form defaults to use the Hook palette, white 8px card, subtle grey border, green top accent, and inline/table email layout.
- 2026-05-13: Validation passed: `cargo fmt --check --all`, `cargo test -p backend` with a 60-second timeout, and `pnpm lint:frontend`.
- 2026-05-13: Reopened task for Aether-style SMTP test connection. Aether performs a real SMTP connection and auth test, does not send mail, and lets an omitted password use the saved encrypted password. Hook extension points are setting use case, admin settings router, API defaults, frontend action, and email settings section.
- 2026-05-13: Added backend SMTP test use case with injected lettre tester, decrypt support for saved SMTP password, API route `/api/admin/settings/smtp/test`, and default API permission binding. `cargo test -p setting` passed.
- 2026-05-13: Wired the frontend SMTP test button to post current unsaved SMTP form values and show success/failure toasts. Replaced the unsupported Iconify name with an icon present in the local icon set, split the email template editor out so frontend files stay below the 300-line limit, and validated with `pnpm lint:frontend` plus `pnpm build:frontend`.
- 2026-05-13: Confirmed registration email suffix restriction had only been implemented as settings/UI, then wired it into `UserService::sign_up` before user creation. Added unit tests for whitelist allow, whitelist reject, and blacklist reject. Validation passed: `cargo fmt --check --all`, `cargo test -p setting`, `cargo test -p user`, and `cargo test -p backend` with 60-second wrappers. `pnpm build:frontend` still prints the existing `Axios error: unauthorized` during static generation but exits successfully.
