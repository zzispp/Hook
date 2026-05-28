# Progress Log

## Session Start

- **Date**: 2026-05-28
- **Task name**: `aether-user-stats`
- **Task dir**: `.codex-tasks/aether-user-stats/`
- **Spec**: See `SPEC.md`
- **Plan**: See `TODO.csv`
- **Environment**: Rust workspace + Next.js frontend

## Context Recovery Block

- **Current milestone**: #2 — Implement backend aggregation and APIs
- **Current status**: IN_PROGRESS
- **Last completed**: #1 — Confirm write path and schema touchpoints
- **Current artifact**: `.codex-tasks/aether-user-stats/TODO.csv`
- **Key context**: Aether `/admin/user-stats` requires leaderboard, selected user summary, selected user cost series, and optional comparison series. User requested baseline-only schema changes.
- **Known issues**: None yet.
- **Next action**: Add baseline aggregate table, storage query/upsert functions, and dashboard admin stats endpoints.

## Milestone 1: Confirm Write Path And Schema Touchpoints

- **Status**: DONE
- **Started**: 2026-05-28
- **Validation**: `rg -n "create_request_record|update_request_record|request_records" crates/storage/src/provider apps/hook_backend/src`
- **What was done**:
  - Located request-record writes in `crates/storage/src/provider/request_record_write.rs`.
  - Located baseline table and index definitions under `apps/hook_backend/src/migration/baseline`.
  - Confirmed existing dashboard crate is the right API boundary for admin overview stats.

## Milestone 2: Implement Backend Aggregation And APIs

- **Status**: IN_PROGRESS
- **Started**: 2026-05-28

## 2026-05-28
- Implemented backend aggregate bucket schema, APIs, and request-record delta sync.
- Implemented standalone admin user stats dashboard route /dashboard/admin/user-stats and menu entry; removed embedding from main dashboard.
- Validation: cargo check --workspace passed, pnpm lint:frontend passed, pnpm build:frontend passed. just test failed in existing llm_proxy::formats::tests::streaming_requests_do_not_route_to_force_non_stream_formats.
