# Backend RBAC Progress

## Recovery

- 任务: Implement backend RBAC/admin/auth middleware/Redis cache.
- 形态: single-full.
- 进度: 7/7.
- 当前: Backend RBAC/admin/Redis implementation validated.
- 文件: `.codex-tasks/20260507-backend-rbac/TODO.csv`.
- 下一步: Inspect crate manifests and implement configuration/type foundations.

## Notes

- Existing backend composition root is `apps/hook_backend`.
- Existing user service keeps business logic in `crates/user`.
- Existing storage uses Toasty models and `db.push_schema()`.
- Existing JWT handling is in `crates/user/src/api/tokens.rs`.
- Step 1 complete: RBAC should be a dedicated crate; config/types/storage/user boundaries remain intact.
- Step 2 complete: added workspace rbac crate skeleton, admin/auth/redis config, shared RBAC/nav snapshot DTOs.
- Step 3 complete: added RBAC Toasty records/store and schema registration. Avoided Toasty generated `path` conflict by naming DB record field `route_path`.
- Step 4 complete: added RBAC service/cache abstractions, storage repository adapter, Redis cache adapter, and service tests.
- Step 5 compile path complete: added RBAC auth middleware, `/api/navbar`, backend Redis wiring, and startup cache rebuild. Route behavioral tests still need expansion later.
- Step 6 complete: config-backed admin is injected into user service, appears in database-paged user listings, can sign in by username/email, and cannot be replaced/deleted or shadowed by normal users.
- Step 7 complete: Redis URL config prefers `redis.url`, RBAC management routes are protected by global auth middleware, role/API/menu mutations rebuild Redis cache, and `just test` passed.
