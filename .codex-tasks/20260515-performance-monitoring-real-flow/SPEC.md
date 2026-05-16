# 真实性能监控统计验证

## 目标

验证性能监控模块能在真实本地后端、真实本地 PostgreSQL、真实上游 LLM 请求链路下统计核心请求与 LLM 业务指标。

## 范围

- 使用环境变量读取真实 provider base/key，不把密钥写入源码、任务文件或结果文件。
- 复用真实 usage concurrency flow 的 DB fixture、provider 加密、admin token 创建和 proxy 请求工具。
- 发起真实 `/v1/chat/completions` 请求，等待 backend 的 performance monitoring worker 写入 minute 快照。
- 用 DB 中同一 minute bucket 的 `request_records` 聚合结果对比快照 JSON 和 admin realtime API。
- 默认执行稳定的非流式真实请求；如需覆盖流式请求数，可设置 `HOOK_PERF_MONITORING_REAL_STREAMS=1` 开启。

## 非目标

- 不自动执行 destructive development migration。
- 不 mock 上游响应。
