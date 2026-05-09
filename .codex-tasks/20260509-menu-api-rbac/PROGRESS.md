# Progress

## Recovery

- 任务: RBAC 菜单绑定 API 重构
- 形态: single-full
- 当前: done
- 文件: `.codex-tasks/20260509-menu-api-rbac/TODO.csv`

## Log

- Created task artifacts and started inventory.
- Completed inventory. Direct role API authorization is present in `role_api_permissions`, `RoleApiBindingInput`, RBAC repository/service/admin routes, frontend role permission tabs, and baseline/default seeds. Target implementation will introduce `menu_api_permissions` and derive API role codes from role-menu plus menu-api bindings.
- Implemented backend type/store/service/repository target shape. API authorization snapshots now derive role codes from enabled menu items bound to APIs plus role menu bindings. Removed role direct API use-case/routes from backend runtime. `cargo check -p rbac` and `cargo check -p storage` passed.
- Added target baseline `menu_api_permissions`, default menu/API mapping, authenticated-base API config for `/api/auth/me` and `/api/navbar`, wallet menu/API seed alignment, and upgrade migration `m20260509_000001_menu_api_permissions`. `cargo check -p backend` passed.
- Refactored frontend RBAC admin UI. Role management now edits menu permissions only; menu management owns API binding through `/api/rbac/menu-items/{id}/apis`. Split menu management table/form/dialog code into focused files, keeping new frontend files under 300 lines. `pnpm lint:frontend` passed.
- Full validation passed: `cargo fmt`, `cargo check -p backend`, `cargo check -p rbac`, `cargo check -p storage`, `just test`, and `pnpm build:frontend`. The first parallel `cargo check -p rbac` hit a target cache race and passed when rerun sequentially. `pnpm build:frontend` prints an existing Axios error during static page generation but exits 0.
- Final cleanup split role management into focused table/state/action/dialog modules. Re-ran `pnpm lint:frontend`, `cargo check -p backend`, `cargo check -p rbac`, `cargo check -p storage`, and `pnpm build:frontend`; all passed. Restored build-generated `apps/hook_frontend/next-env.d.ts`.
