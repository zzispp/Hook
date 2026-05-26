# Progress

## 2026-05-26

- Started implementation from accepted plan.
- Confirmed applicable AGENTS files: repo root and `apps/hook_backend`.
- Added user group storage, baseline seed, service/API wiring, billing group visibility binding, default registration group setting, and token visibility policy.
- Added admin user group management UI, user form/group badge integration, billing group user-group multi-select, system settings default user group selector, menu/default API permissions, and backend i18n seed keys.
- Validation passed:
  - `jq empty apps/hook_backend/src/migration/defaults/i18n/admin.cn.json apps/hook_backend/src/migration/defaults/i18n/admin.en.json`
  - `pnpm lint:frontend`
  - `pnpm build:frontend`
  - `cargo fmt --all`
  - `cargo check -p storage -p types -p constants -p user -p group -p setting -p api_token -p backend -p provider -p rbac -p card_code --all-targets`
  - `just test`
  - `cargo clippy -p storage -p types -p constants -p user -p group -p setting -p api_token -p backend -p provider -p rbac -p card_code -p recharge --all-targets --no-deps -- -D warnings`
- Browser route check: `http://localhost:8082/dashboard/admin/user-groups/` is served by the frontend dev server; page data loading shows `Axios error: Network Error` because the local backend API is not running in this session.
- `cargo clippy --workspace --all-targets -- -D warnings` is blocked by existing `crates/proxy` style lints unrelated to this user-group change (`derivable_impls`, `unnecessary_lazy_evaluations`, `needless_question_mark`, `collapsible_if`).
