# Progress

## 2026-06-26

- 已确认后端当前问题有两个：响应头时间计算早但落库晚；`first_sse_event_time_ms` 目前实际上记录的是首个上游分片。
- 已确认 `first_output_time_ms` 的严格输出语义可直接保留，不需要改检测口径。
- 已确认 routing / performance / 成本 / 用户统计的边界：本次只改请求记录层和 Admin 展示链路。
- 已完成后端实现：
  - 成功流式请求在拿到上游成功响应头后立即写入 `response_headers_time_ms`
  - `first_sse_event_time_ms` 改为首个有效 SSE `data:` 事件，不再把 keepalive / 空 `data:` / `[DONE]` 记为首字
  - `first_output_time_ms` 保持现有严格输出语义
- 已完成前端实现：
  - `response_headers / first_sse_event / first_output` 展示不再回退到 `first_byte_time_ms`
  - expanded timing、详情抽屉、trace timeline、usage records 统一成“响应头 / 首字 / 首 Token / 总耗时”语义
  - 中英文 admin seed 文案已同步更新
- 已完成验证：
  - `cargo test -p hook_backend llm_proxy::proxy::stream_transport -- --nocapture`
  - `CI=true corepack pnpm lint:frontend`
  - `CI=true corepack pnpm build:frontend`
