# Progress

## 2026-05-25

- Investigated `scheduled_tasks` and `scheduled_task_runs` warnings.
- Confirmed the warned `Model` and `Relation` types are SeaORM table mappings used by `SchedulerStore`; deleting them would break scheduled task persistence.
- Changed `crates/storage/src/scheduler/mod.rs` to expose the `entities` module directly, matching the actual persistence boundary.
- Changed `crates/storage/src/scheduler/record.rs` to reuse `super::entities` instead of defining a private path-mounted entity module.
- Validation passed: `timeout 60 cargo check -p storage`, `timeout 60 cargo build -p storage`, and `timeout 60 cargo build -p scheduler` completed without warnings.
