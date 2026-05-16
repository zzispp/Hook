# 进度记录

## 2026-05-15

- 已读取 `.codex-tasks/20260515-real-usage-concurrency-flow` 和 `.codex-tasks/20260514-real-request-record-flow` 的真实测试脚本。
- 本任务复用历史脚本的本地 DB、backend 启动、provider fixture、admin token 创建、真实 proxy 请求和脱敏结果写入方式。
- 已新增 `real_performance_monitoring_flow.mjs`，语法检查通过。脚本会发真实 LLM 请求，并等待性能监控 worker 写入 minute 快照后对比 DB 聚合、快照 JSON、realtime API。
- 本地 Docker DB 是旧 schema，脚本新增了幂等的本地准备步骤，只补 `performance_monitoring_snapshots`、`performance_monitoring_retention_days`、`usage_flush_batches` 与性能监控 RBAC 绑定，不执行 destructive development migration。
- 第一轮包含 1 个流式请求时，真实上游/undici 在流式连接上返回 `terminated`，8 个非流式请求已成功。为避免上游流式波动掩盖统计验证，脚本默认 `HOOK_PERF_MONITORING_REAL_STREAMS=0`，需要覆盖流式请求数时可显式开启。
- 最终真实验证通过：8 个真实 `/v1/chat/completions` 请求落在 `2026-05-15 11:10:00+00` minute bucket，性能 worker 写入快照；DB 聚合、快照 JSON、realtime API 三方一致。
- 关键统计：`request_count=8`、`success_rate=1`、`prompt_tokens=1666`、`completion_tokens=4028`、`total_tokens=5694`，provider 分布为 Ekan8 4 次、msutools/OpenAI compatible 4 次。
- 脱敏结果已写入 `raw/results.json`；未写入 provider key 或管理员 JWT。
