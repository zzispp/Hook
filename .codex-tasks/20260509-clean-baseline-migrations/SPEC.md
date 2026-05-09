# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- 将开发阶段的数据库目标状态折叠为唯一 baseline migration。
- baseline 直接创建当前业务需要的 RBAC、菜单 API 绑定、钱包和钱包流水表结构。
- 本地 PostgreSQL 直接重建，不保留旧数据库升级兼容路径。

## Non-Goals

- 不保留旧 DB 兼容迁移。
- 不写静默兜底、mock 或降级逻辑。
- 不改变当前已实现的业务权限模型。

## Constraints

- 用户要求使用 CNY 钱包。
- 用户角色可见钱包中心，管理员不可见钱包中心。
- API 权限由菜单绑定，角色只绑定菜单。
- 操作本地 PostgreSQL 前必须确认目标为 localhost。

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + Next.js frontend
- **Package manager**: pnpm / cargo
- **Test framework**: cargo test
- **Build command**: `cargo check`
- **Existing test count**: existing workspace tests

## Risk Assessment

- [x] External dependencies (APIs, services) — local PostgreSQL target confirmed as `localhost:5433/postgres`.
- [x] Breaking changes to existing code — intended because development DB has no compatibility requirement.
- [x] Large file generation — not applicable.
- [x] Long-running tests — repository has `just test` 60-second timeout wrapper.

## Deliverables

- Clean migration registration with baseline only.
- Baseline table/identifier/index/seed definitions matching current entities.
- Removed obsolete incremental migration files.
- Local DB rebuilt from clean baseline.

## Done-When

- [ ] `cargo fmt --all` passes.
- [ ] backend/storage/rbac/wallet checks pass.
- [ ] local PostgreSQL migrates from empty schema using baseline only.

## Final Validation Command

```bash
cargo fmt --all && cargo check -p backend && cargo check -p storage && cargo check -p rbac && cargo test -p wallet
```
