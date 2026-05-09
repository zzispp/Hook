# Progress Log

---

## Session Start

- **Date**: 2026-05-09 15:06 CST
- **Task name**: `20260509-clean-baseline-migrations`
- **Task dir**: `.codex-tasks/20260509-clean-baseline-migrations/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (4 milestones)
- **Environment**: Rust workspace / SeaORM migration / cargo test

---

## Context Recovery Block

- **Current milestone**: #4 — 运行验证
- **Current status**: DONE
- **Last completed**: #4 — 运行验证
- **Current artifact**: `apps/hook_backend/src/migration/m20260508_000001_create_baseline/*`
- **Key context**: 当前代码目标状态是角色只绑菜单、菜单绑 API；钱包是 CNY 用户钱包中心，管理员默认不绑定钱包中心。
- **Known issues**: none.
- **Next action**: report completion.

---

## Milestone 1: 梳理目标 schema 和 seed

- **Status**: DONE
- **Started**: 15:03
- **Completed**: 15:06
- **What was done**:
  - 读取 baseline、增量迁移、defaults、storage entities 和本地配置。
- **Key decisions**:
  - Decision: 只保留 baseline migration。
  - Reasoning: 用户明确开发阶段不考虑旧 DB 兼容，本地 DB 可以直接改。
  - Alternatives considered: 保留增量迁移但清理内容；这仍会保留旧库升级路径，不符合要求。
- **Problems encountered**:
  - Problem: baseline 已部分混入新菜单 API 设计但缺审计字段和钱包表。
  - Resolution: 将目标状态直接补齐到 baseline。
  - Retry count: 0
- **Validation**: context inspection → exit 0
- **Files changed**:
  - `.codex-tasks/20260509-clean-baseline-migrations/*` — taskmaster 记录。
- **Next step**: Milestone 2 — 重写 baseline 并删除兼容迁移

## Milestone 2: 重写 baseline 并删除兼容迁移

- **Status**: DONE
- **Started**: 15:06
- **Completed**: 15:17
- **What was done**:
  - `Migrator` 改为只注册 baseline。
  - baseline 直接创建 RBAC audit columns、`menu_api_permissions`、`wallets`、`wallet_transactions`。
  - 删除 RBAC 时间字段、钱包、钱包用户访问、菜单 API 绑定相关旧增量迁移。
- **Key decisions**:
  - Decision: 新增 `wallet_tables.rs` 承载钱包建表逻辑。
  - Reasoning: 钱包表结构较独立，拆分后文件长度和职责都更干净。
- **Problems encountered**:
  - Problem: SeaQuery `foreign_key` API 需要 `&mut ForeignKeyCreateStatement`。
  - Resolution: 改为局部 mutable FK statement 后传入。
  - Retry count: 0
- **Validation**: `cargo check -p backend` → exit 0
- **Files changed**:
  - `apps/hook_backend/src/migration/mod.rs` — 只注册 baseline。
  - `apps/hook_backend/src/migration/m20260508_000001_create_baseline/*` — 折叠最终 schema/seed。
  - `apps/hook_backend/src/migration/m20260508_000002_add_rbac_timestamps.rs` — deleted。
  - `apps/hook_backend/src/migration/m20260508_000003_add_wallets.rs` — deleted。
  - `apps/hook_backend/src/migration/m20260508_000003_add_wallets/rbac_seed.rs` — deleted。
  - `apps/hook_backend/src/migration/m20260508_000004_wallet_user_only_access.rs` — deleted。
  - `apps/hook_backend/src/migration/m20260509_000001_menu_api_permissions.rs` — deleted。
- **Next step**: Milestone 3 — 重建本地 PostgreSQL 并跑迁移

## Milestone 3: 重建本地 PostgreSQL 并跑迁移

- **Status**: DONE
- **Started**: 15:14
- **Completed**: 15:17
- **What was done**:
  - 确认配置为 `localhost:5433/postgres`。
  - 执行 `DROP SCHEMA public CASCADE; CREATE SCHEMA public;`。
  - 执行 `cargo run -p backend -- --config config/config.yaml migration fresh`。
- **Key decisions**:
  - Decision: 直接重建 public schema。
  - Reasoning: 用户明确开发阶段不需要旧 DB 兼容，清库能验证 baseline 本身。
- **Problems encountered**:
  - Problem: 初次运行 migration history 显示为 `mod`。
  - Resolution: 手写 `MigrationName` 返回 `m20260508_000001_create_baseline` 后重跑。
  - Retry count: 0
- **Validation**: local PostgreSQL migration fresh → exit 0
- **Files changed**:
  - `apps/hook_backend/src/migration/m20260508_000001_create_baseline/mod.rs` — 显式 migration name。
- **Next step**: Milestone 4 — 运行验证

## Milestone 4: 运行验证

- **Status**: DONE
- **Started**: 15:17
- **Completed**: 15:17
- **What was done**:
  - DB 断言确认表、索引、角色菜单绑定、菜单 API 绑定符合目标。
  - 执行 Rust 格式化、check、wallet test 和全仓库 test。
- **Key decisions**:
  - Decision: `AUTHENTICATED_API_CODES` 仅测试编译。
  - Reasoning: 构建时不再用于旧迁移验证，但测试仍用它保护 auth base API 不被菜单绑定。
- **Problems encountered**:
  - Problem: none.
  - Resolution: none.
  - Retry count: 0
- **Validation**: `just test` → exit 0
- **Files changed**:
  - `.codex-tasks/20260509-clean-baseline-migrations/*` — closeout records。

## Final Summary

- **Total milestones**: 4
- **Completed**: 4
- **Failed + recovered**: 0
- **External unblock events**: 0
- **Total retries**: 0
- **Files created**: 4
- **Files modified**: migration baseline and task records
- **Key learnings**:
  - SeaORM directory `mod.rs` migrations need explicit `MigrationName` when the desired migration name is the module directory.
