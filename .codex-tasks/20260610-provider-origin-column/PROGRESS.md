# Progress

## 2026-06-10
- Started Single Task tracking for provider origin column.
- Added `provider_origin` as a persisted provider field.
- Manual provider creation writes `manual`; quick import storage writes `quick_import`.
- Provider list now renders an "添加方式 / Creation method" column.
- Validation passed:
  - `cargo fmt --check`
  - `timeout 60 cargo check -p types -p storage -p provider -p hook_backend`
  - `timeout 60 cargo test -p storage --test provider_quick_import -- --nocapture`
  - `timeout 60 cargo test -p storage --test provider_list_filters -- --nocapture`
  - `timeout 60 cargo test -p storage --test provider_create_group_binding -- --nocapture`
  - `timeout 60 cargo test -p provider quick_import -- --nocapture`
  - `jq empty apps/hook_backend/src/migration/defaults/i18n/admin.cn.json apps/hook_backend/src/migration/defaults/i18n/admin.en.json`
  - `pnpm lint:frontend`
  - `pnpm build:frontend`
