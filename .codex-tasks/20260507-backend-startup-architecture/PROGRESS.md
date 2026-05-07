# Progress

## Recovery

- 任务: 后端入口与架构边界收拢
- 形态: single-full
- 进度: 5/5
- 当前: Done
- 文件: `.codex-tasks/20260507-backend-startup-architecture/TODO.csv`
- 下一步: None.

## Notes

- `apps/hook_backend` is the intended composition root according to its local `AGENTS.md`.
- `main.rs` currently mixes entrypoint, CLI parsing, runtime wiring, schema push, migration passthrough, authorization config mapping, and tests.
- `crates/rbac/src/api/state.rs` and `crates/rbac/src/api/auth.rs` directly depend on the `user` crate, which makes RBAC API know user token/user lookup details. The cleaner boundary is backend-level auth middleware or an RBAC-defined auth subject port injected by backend.
- Implemented `apps/hook_backend/src/commands.rs` for backend command dispatch.
- Implemented `apps/hook_backend/src/startup.rs` for runtime wiring and app construction.
- Implemented `apps/hook_backend/src/auth.rs` for backend-level auth middleware that coordinates user token validation and RBAC authorization.
- Removed the `user` dependency from the `rbac` crate.
