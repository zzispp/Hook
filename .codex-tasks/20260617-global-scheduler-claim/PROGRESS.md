# Progress Log

---

## Session Start

- **Date**: 2026-06-17
- **Task name**: `20260617-global-scheduler-claim`
- **Task dir**: `.codex-tasks/20260617-global-scheduler-claim/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (5 milestones)
- **Environment**: Rust workspace / scheduler + storage crates / Cargo tests

---

## Context Recovery Block

- **Current milestone**: #5 — Run final validation
- **Current status**: DONE
- **Last completed**: #4 — Add or update tests for multi-instance due claims
- **Current artifact**: `TODO.csv`
- **Key context**: The production incident showed blue and green backend instances both running scheduler loops. Current advisory lock prevents simultaneous execution only, not duplicate due windows. New schema additive must run before older additives that query `scheduled_tasks::Entity`.
- **Known issues**: None.
- **Next action**: Ready for user review.

---

## Milestone 1: Map scheduler storage and migration impact

- **Status**: DONE
- **Started**: 2026-06-17
- **Completed**: 2026-06-17
- **What was done**:
  - Inspected scheduler runtime, worker, storage repository, entity definitions, baseline schema, and additive migration order.
- **Key decisions**:
  - Decision: implement a shared database due-time claim instead of relying on local timers or execution-only advisory locks.
  - Reasoning: blue/green instances can run at different offsets; locking only execution does not prevent duplicate due windows.
  - Alternatives considered: leader-only scheduler and longer advisory locks; both add broader operational coupling than per-task due claims.
- **Problems encountered**:
  - Problem: an existing additive named `scheduled_task_next_run_additive` does not add next-run schema.
  - Resolution: add a new explicit global claim additive and run it before existing additives that use the scheduler entity.
  - Retry count: 0
- **Validation**: `rg -n "scheduled_tasks|SchedulerRuntime|dispatch_task|mark_task_started|task_record" crates/scheduler crates/storage apps/hook_backend/src/migration` -> exit 0
- **Files changed**:
  - `.codex-tasks/20260617-global-scheduler-claim/*` — task tracking artifacts.
- **Next step**: Milestone 2 — Add global claim schema and storage API

---

## Milestone 2: Add global claim schema and storage API

- **Status**: DONE
- **Started**: 2026-06-17
- **Completed**: 2026-06-17
- **What was done**:
  - Added `next_run_at`, `locked_until`, and `locked_by` to the scheduled task entity and baseline schema.
  - Added an early additive migration that backfills `next_run_at`, makes it non-null, and creates the due-claim index.
  - Added `claim_due_task` with `FOR UPDATE SKIP LOCKED` and `finish_claimed_task_run` with `locked_by` fencing.
  - Added storage tests for atomic due claim SQL and stale-owner rejection.
- **Key decisions**:
  - Decision: `locked_by` is the fencing token; finish updates are ignored if another owner has reclaimed the row.
  - Reasoning: a crashed or slow instance must not overwrite a later claim's task summary.
- **Problems encountered**:
  - Problem: `ScheduledTaskClaim` could not derive `Eq` because the SeaORM model only derives `PartialEq`.
  - Resolution: removed `Eq` from the claim type and imported `FromQueryResult` for raw SQL model hydration.
  - Retry count: 0
- **Validation**: `timeout 60s cargo test -p storage --test scheduler_claims` -> passed
- **Files changed**:
  - `crates/storage/src/scheduler/*`
  - `apps/hook_backend/src/migration/*`
  - `crates/storage/tests/scheduler_claims.rs`
- **Next step**: Milestone 3 — Refactor scheduler runtime to use claim API

---

## Milestone 3: Refactor scheduler runtime to use claim API

- **Status**: DONE
- **Started**: 2026-06-17
- **Completed**: 2026-06-17
- **What was done**:
  - Removed the runtime `next_runs` map and API response overlay.
  - Changed scheduler rescheduling to use `scheduled_tasks.next_run_at` and `locked_until`.
  - Changed dispatch so the worker claims the due row before creating any run or skip record.
  - Kept advisory locks as execution protection after the database claim.
- **Key decisions**:
  - Decision: DelayQueue is only a local wake-up mechanism; database `next_run_at` is the source of truth.
  - Reasoning: local next-run maps diverge across blue/green instances and recreate the duplicate due-window problem.
- **Problems encountered**:
  - Problem: local running skip could have bypassed the claim if checked before storage.
  - Resolution: dispatch now claims first, then records local skip under the claim owner.
  - Retry count: 0
- **Validation**: `timeout 60s cargo test -p scheduler` -> passed
- **Files changed**:
  - `crates/scheduler/src/runtime/service.rs`
  - `crates/scheduler/src/runtime/worker.rs`
  - `crates/scheduler/src/runtime/*_tests.rs`
- **Next step**: Milestone 4 — Add or update tests for multi-instance due claims

---

## Milestone 4: Add or update tests for multi-instance due claims

- **Status**: DONE
- **Started**: 2026-06-17
- **Completed**: 2026-06-17
- **What was done**:
  - Added storage claim tests for atomic SQL and stale owner rejection.
  - Updated scheduler tests for DB-backed next attempt timing.
  - Added worker tests for claim-none no-run, local-running skip after claim, and advisory-lock skip after claim.
- **Key decisions**:
  - Decision: tests assert SQL/order where MockDatabase cannot run true concurrent row locks.
  - Reasoning: the atomic behavior is delegated to Postgres `FOR UPDATE SKIP LOCKED`; unit tests should lock down generated SQL and runtime side effects.
- **Problems encountered**:
  - Problem: no real Postgres integration harness is present in the repo for concurrent scheduler claims.
  - Resolution: used storage SQL assertions plus worker side-effect tests; final validation still uses crate-level Cargo tests.
  - Retry count: 0
- **Validation**:
  - `timeout 60s cargo test -p scheduler` -> passed
  - `timeout 60s cargo test -p storage --test scheduler_claims` -> passed
- **Files changed**:
  - `crates/scheduler/src/runtime/service_tests.rs`
  - `crates/scheduler/src/runtime/worker_tests.rs`
  - `crates/storage/tests/scheduler_claims.rs`
- **Next step**: Milestone 5 — Run final validation

---

## Milestone 5: Run final validation

- **Status**: DONE
- **Started**: 2026-06-17
- **Completed**: 2026-06-17
- **What was done**:
  - Ran rustfmt.
  - Ran scheduler and storage test suites.
  - Ran workspace `just check`.
- **Key decisions**:
  - Decision: final validation stayed on backend/Rust targets because the frontend is unaffected.
  - Reasoning: the change is limited to Rust migration, storage, and scheduler runtime code.
- **Problems encountered**:
  - None.
  - Retry count: 0
- **Validation**:
  - `cargo fmt` -> passed
  - `timeout 60s cargo test -p scheduler -p storage` -> passed
  - `just check` -> passed
- **Files changed**:
  - See git diff/status.
- **Next step**: Complete.
