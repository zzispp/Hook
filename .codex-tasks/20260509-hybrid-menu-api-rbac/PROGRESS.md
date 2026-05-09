# Progress Log

---

## Session Start

- **Date**: 2026-05-09 15:20 CST
- **Task name**: `20260509-hybrid-menu-api-rbac`
- **Task dir**: `.codex-tasks/20260509-hybrid-menu-api-rbac/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (5 milestones)
- **Environment**: Rust workspace / Next.js

---

## Context Recovery Block

- **Current milestone**: #1 — 梳理现有 RBAC 链路
- **Current status**: IN_PROGRESS
- **Last completed**: none
- **Current artifact**: `crates/rbac`, `crates/storage/src/rbac`, `apps/hook_frontend/src/sections/admin`
- **Key context**: 当前实现已经是菜单 API 模式，刚清理成 baseline-only；现在要恢复角色 API 直授权并与菜单 API 并集授权。
- **Known issues**: 旧 `role_api_permissions` entity 已删除，需要干净重建。
- **Next action**: 读取 RBAC repository/service/types/frontend 状态。
2026-05-09T07:29:09Z - Resumed hybrid menu/API RBAC task. Confirmed `RolePermissionBindingInput` and `ApiMenuBindingInput` already exist in `crates/types/src/rbac.rs`; storage/service/frontend still reference menu-only role bindings.
2026-05-09T07:32:24Z - Added baseline/storage `role_api_permissions` support, API-menu replacement storage methods, and role permission replacement storage method.
2026-05-09T07:42:17Z - Replaced role menu-only service/API with role permission bindings, added API-menu HTTP endpoints, merged role API bindings into permission snapshot, and split oversized RBAC files. `cargo check -p backend` passes.
2026-05-09T07:47:09Z - Updated frontend actions, API management form, and role permission dialog for menu/API hybrid bindings. `pnpm --dir apps/hook_frontend lint` passes.
