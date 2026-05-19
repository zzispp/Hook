# Progress Log

## Session Start

- **Date**: 2026-05-19 15:46 CST
- **Task name**: stream idle usage fix
- **Task dir**: `.codex-tasks/20260519-stream-idle-usage-fix/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv
- **Environment**: Rust workspace / just test

## Context Recovery Block

- **Current milestone**: #4 — 运行后端验证
- **Current status**: DONE
- **Last completed**: #3 — 验证 EOF missing usage 估算
- **Current artifact**: `TODO.csv`
- **Key context**: Provider-level `stream_idle_timeout_seconds` is in baseline/types/storage/cache/candidate and relay. EOF with no output now estimates request usage instead of leaving `missing_usage`.
- **Known issues**: Working tree already has unrelated user changes; do not revert them.
- **Next action**: Final response to user.

## Milestone 1: 梳理受影响结构

- **Status**: DONE
- **Started**: 15:46
- **Completed**: 15:46
- **What was done**:
  - Located timeout fields across provider types, storage entities, cache snapshot, candidate build, and stream relay.
  - Confirmed existing usage estimation covers EOF with output delta and does not estimate empty output.
- **Validation**: `rg -n "stream_first_byte_timeout_seconds|request_timeout_seconds" apps crates` → exit 0
- **Next step**: Milestone 2 — 实现显式 stream idle timeout

## Milestone 2: 实现显式 stream idle timeout

- **Status**: DONE
- **Started**: 15:46
- **Completed**: 15:57
- **What was done**:
  - Added `stream_idle_timeout_seconds` to provider baseline schema, API types, storage entity/patch, scheduling snapshot, and proxy candidate.
  - Added relay idle deadline computation. Keepalive still fires before the idle deadline; once idle timeout is reached, the stream records `failed/upstream_timeout`.
- **Validation**: `cargo test -p backend upstream_wait --no-default-features` -> exit 0
- **Next step**: Milestone 3 — 验证 EOF missing usage 估算

## Milestone 3: 验证 EOF missing usage 估算

- **Status**: DONE
- **Started**: 15:57
- **Completed**: 15:57
- **What was done**:
  - Kept output-delta estimation unchanged.
  - Added request-body-only usage estimation with `usage_source=estimated_from_request_body` for EOF streams with no provider usage and no output delta.
- **Validation**: `cargo test -p backend empty_output_can_estimate_request_usage_for_billing --no-default-features` -> exit 0
- **Next step**: Milestone 4 — 运行后端验证

## Milestone 4: 运行后端验证

- **Status**: DONE
- **Started**: 15:58
- **Completed**: 16:02
- **What was done**:
  - Ran focused backend tests, backend full unit tests, Rust workspace check, frontend lint, and repository test wrapper.
- **Validation**:
  - `cargo test -p backend --no-default-features` -> exit 0
  - `just check` -> exit 0
  - `pnpm lint:frontend` -> exit 0
  - `just test` -> exit 0
- **Next step**: Final response.

## Final Summary

- Added explicit stream idle timeout configuration and relay enforcement.
- Added request-body usage estimation for EOF streams with no provider usage and no output delta.
- Updated baseline provider schema, provider types, scheduling snapshot, proxy candidate, admin form, and i18n seeds.
- Did not modify production DB or production request records.
