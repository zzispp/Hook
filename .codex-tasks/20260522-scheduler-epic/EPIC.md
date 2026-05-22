# Scheduler Epic

## Goal

Split scheduled tasks into a dedicated Rust crate, unify scheduling configuration in a DB-backed admin dashboard, and remove scheduler-related controls from system settings.

## Delivery Boundary

- Add `crates/scheduler` with `ScheduledTask`, registry, DB-backed runtime, and `tokio-util::time::DelayQueue` scheduling.
- Move existing manageable background jobs into registered task implementations.
- Store task enablement, intervals, task-specific parameters, and execution history in scheduler tables.
- Add admin APIs and Dashboard tabs for task list and execution records.
- Remove scheduler enable/interval/retention fields from system settings and the frontend settings form.
- Keep `llm_proxy` usage flush as an internal critical loop, not a user-disableable scheduled task.

## Runtime Semantics

- One central scheduler loop owns the `DelayQueue`.
- Each expired task execution is launched via independent `tokio::spawn`.
- Different tasks can run in parallel.
- The same task does not re-enter while a previous execution is still running.
- Scheduler API updates notify the runtime through a reload channel so `DelayQueue` entries are removed and reinserted from the latest DB state.

## Validation Gates

- Rust checks/tests for scheduler, storage, migrated task behavior, and API handlers.
- Frontend lint/build for Dashboard and settings removal.
- No silent fallback behavior; missing or invalid task config must surface explicit errors.
