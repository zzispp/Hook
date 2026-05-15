# Progress Log

## Context Recovery Block

- **Current milestone**: #1 — Add failing tests for snapshot-backed request record policy
- **Current status**: IN_PROGRESS
- **Last completed**: none
- **Current artifact**: `TODO.csv`
- **Key context**: 计划把 request record policy 字段并入 `SchedulingSnapshot`，让审计路径只依赖 Redis snapshot。
- **Known issues**: `SchedulingSnapshot` 命名仍保留，但内容会扩大到 runtime policy。
- **Next action**: 补失败测试并跑 `cargo test -p backend llm_proxy::request_record_policy -- --nocapture`

