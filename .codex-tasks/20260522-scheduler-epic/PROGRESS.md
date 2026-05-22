# Scheduler Epic Progress

## 2026-05-22

- Added `crates/scheduler` with registry, `DelayQueue` runtime, reload channel, non-reentrant per-task execution, admin API router, and DB-backed runtime config.
- Added `scheduled_tasks` and `scheduled_task_runs` baseline tables, indices, storage models, repository, and shared scheduler DTOs.
- Migrated backend scheduled jobs into registered tasks: `api_token_cleanup`, `request_record_cleanup`, `request_record_stale_sweep`, `performance_monitoring_snapshot`, `performance_monitoring_cleanup`.
- Switched backend startup from old standalone worker loops to scheduler runtime registration.
- Removed scheduler-related fields from `system_settings` types, storage, baseline schema, seed, validation, and frontend system settings UI.
- Added admin menu/API seeds and backend i18n seed entries for scheduled tasks.
- Added frontend `/dashboard/admin/scheduled-tasks` page with task list + run record tabs, dynamic task config editing, enable/disable toggles, and pagination.
- Deleted legacy worker files replaced by scheduler runtime.
- Validation so far: `cargo check -p scheduler`, `cargo check -p setting`, `cargo check -p backend`, `just check`, `pnpm build:frontend` passed; `timeout 60 just test` hit the repository timeout wrapper before completion.
- Remaining cleanup: `apps/hook_backend/src/scheduled_tasks.rs` still exceeds the repo file-length limit and should be split further.
