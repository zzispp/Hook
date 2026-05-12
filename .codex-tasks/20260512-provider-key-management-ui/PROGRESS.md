# Progress Log

## Context Recovery Block

- **Current milestone**: #2 — Add provider key update delete backend API
- **Current status**: IN_PROGRESS
- **Last completed**: #1 — Inspect current Hook and Aether key management
- **Current artifact**: `TODO.csv`
- **Key context**: Hook has create/list provider key APIs and a `ProviderApiKeyUpdate` type, but no update/delete routes or storage methods yet.
- **Known issues**: Hook does not expose decrypted provider keys to the frontend, so full-key copy is not implemented as fake UI.
- **Next action**: Wire update/delete through storage, provider service, routes, actions, and then replace the drawer key list.



## 2026-05-12 密钥管理完成
- 增加 provider key PATCH/DELETE 后端链路和 seed 权限。
- Provider drawer 密钥管理改为 aether 风格紧凑列表，支持编辑、启禁、删除。
- 本地 DB 已同步 provider_keys_update/provider_keys_delete 和新增 admin i18n。
- 验证：cargo fmt --all；cargo check -p backend；定向 eslint；pnpm --filter hook_frontend build；git diff --check。
