# Progress Log

## Session Start

- **Date**: 2026-05-13 15:00
- **Task name**: `20260513-request-record-redesign`
- **Task dir**: `.codex-tasks/20260513-request-record-redesign/`
- **Spec**: See `EPIC.md`
- **Plan**: See `SUBTASKS.csv` (8 child tasks)
- **Environment**: Rust workspace / Next.js frontend / Postgres / pnpm

## Completion

- **Completed at**: 2026-05-13 18:25
- **Current milestone**: #3-#8 全部完成
- **Current status**: DONE
- **Current artifact**: `.codex-tasks/20260513-request-record-redesign/SUBTASKS.csv`
- **Delivered changes**:
  - `request_records` 固定为 client 视角主记录，`request_candidates` 固定为 provider attempt 记录。
  - HTTP 流式与 WS 现在会在客户端断开时写入 `cancelled / 499 / client_disconnected`，不再把断连误记成 `success` 或遗留 `streaming`。
  - 非 2xx 错误会提取具体错误片段，详情页同时展示 client payload 和 selected attempt 的 provider payload。
  - cleanup、baseline seed、系统设置默认值和 admin i18n 已对齐新的双视角请求记录语义。
- **Validation**:
  - `cargo test -p storage --test provider_request_records -- --nocapture`
  - `cargo test -p storage --test provider_request_candidates -- --nocapture`
  - `cargo test -p backend llm_proxy -- --nocapture`
  - `cargo check -p backend`
  - `just check`
  - `just test`
  - `pnpm lint:frontend`
