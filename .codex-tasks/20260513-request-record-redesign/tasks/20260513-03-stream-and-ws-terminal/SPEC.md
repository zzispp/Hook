# Single Task Spec

## Goal

- 修正 HTTP 流式与 WS 的终态记录，补齐取消归因骨架。

## Scope

- `apps/hook_backend/src/llm_proxy/proxy/stream_transport/**`
- `apps/hook_backend/src/llm_proxy/ws/**`
- `apps/hook_backend/src/llm_proxy/audit.rs`

## Done-When

- 客户端提前断开不会再被误记为 success
- 流式和 WS 都能写入最终主状态
- `termination_origin / termination_reason / stream_end_reason` 开始参与写入
