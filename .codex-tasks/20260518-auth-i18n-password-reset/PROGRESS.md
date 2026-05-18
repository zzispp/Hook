# Progress

## Recovery
任务: 登录注册多语言、找回密码、邮件模板多语言
形态: single-full
进度: 0/5
当前: Inspect auth i18n email reset architecture
文件: .codex-tasks/20260518-auth-i18n-password-reset/TODO.csv

## Log
- Created task truth files. Existing unrelated workspace changes are preserved.

- Step 1 done: auth pages are hardcoded, i18n validation allows only admin namespace, existing SMTP settings/templates are global and not language-aware.
- Step 2 started: backend password reset and multilingual auth email templates.
- Scope update: user requested a base system setting switch for password reset. Backend must require this switch plus enabled/complete email config before sending reset email.
- Step 2 validation passed: `cargo test -p user` passed with password reset API/service/storage wiring in place.
- Step 3 done: added `password_reset_enabled` to system settings, enforced email prerequisites in setting service, wired admin base switch, and re-ran `cargo test -p setting` plus `cargo test -p user`.
- Step 4 done: auth UI now loads backend `auth` resources, login/register copy and validation use translations, and forgot/reset password pages are routed.
- Step 5 done: cn/en auth seeds include UI text and multilingual password reset templates; translation management can switch between admin/auth namespaces.
- Step 6 done: `cargo test -p user`, `cargo test -p setting`, `cargo check --workspace`, `pnpm lint:frontend`, and `pnpm build:frontend` passed.
