# 进度记录

## 2026-05-15

- 已确认当前没有 brave/context7 MCP 可直接调用资源；使用 docs.rs 官方 sysinfo 文档确认 API。
- 策略：backend 层新增 OS collector，storage 继续只负责请求/LLM聚合与快照持久化。
- 已新增 backend 真实 OS collector：sysinfo 采 CPU/load/memory/disk/process/FD/thread/network bytes，netstat2 采 TCP total/ESTABLISHED/CLOSE_WAIT/TIME_WAIT/SYN。
- collector 通过 `spawn_blocking` 执行真实 OS 读取，realtime API 与快照 worker 共享同一个实例；worker 写快照时注入 host/network，realtime 返回最新快照时用当前 OS 指标覆盖展示。
- 已移除 storage 聚合里的默认 fallback 路径，worker 必须显式传入 `SystemMetricsSnapshot`，避免静默写入 unsupported。
- 验证通过：`cargo fmt --check`、`cargo check -p backend`、`cargo test -p backend performance_monitoring -- --nocapture`、`cargo test -p storage performance_monitoring -- --nocapture`。
- 已在 TCP socket 读取失败路径增加显式 warn 日志，随后重新跑过上述格式、编译、backend/storage 测试。
