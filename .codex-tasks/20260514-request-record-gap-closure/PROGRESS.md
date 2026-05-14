# Progress Log

## Session Start

- **Date**: 2026-05-14
- **Task name**: `20260514-request-record-gap-closure`
- **Task dir**: `.codex-tasks/20260514-request-record-gap-closure/`
- **Spec**: See `SPEC.md`
- **Plan**: See `TODO.csv`
- **Environment**: Rust workspace / Next.js frontend / Postgres / pnpm

## Completion

- **Completed at**: 2026-05-14 09:47
- **Current status**: DONE
- **Current artifact**: `.codex-tasks/20260514-request-record-gap-closure/TODO.csv`
- **Delivered changes**:
  - `request_candidates` 新增 `skip_reason`、`error_code`、`error_param`，并把调度终态语义切换为 `scheduled/skipped`。
  - 非 2xx provider 错误现在会同时提取 message/code/param；前端 trace 可展示 skip reason 与 provider error code/param。
  - payload retention 改为压缩保留，详情查询会透明解压；系统设置文案同步改成“压缩后保留直到总保留期删除”。
  - 新增 stale request sweep：周期性终结超时 `pending/streaming` 主记录，并把活动 candidate 标记为 `failed`、未执行 candidate 标记为 `skipped`。
- **Validation**:
  - `cargo test -p storage --test provider_request_candidates -- --nocapture`
  - `cargo test -p storage --test provider_request_records -- --nocapture`
  - `cargo test -p storage --test provider_request_housekeeping -- --nocapture`
  - `cargo test -p backend llm_proxy -- --nocapture`
  - `cargo test -p proxy request_candidate -- --nocapture`
  - `cargo check -p storage`
  - `cargo check -p backend`
  - `just check`
  - `just test`
  - `pnpm lint:frontend`
