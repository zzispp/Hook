# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- 将 request record policy 从 `system_settings` 的逐请求读取改为 runtime snapshot 恢复。
- 保持现有 body/header 记录与截断语义不变。

## Non-Goals

- 不在本子任务内改动请求记录表结构。

## Constraints

- 继续使用现有 `LlmProxyCache` 刷新链路。
- 审计路径失败必须显式返回错误。

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024
- **Package manager**: cargo / just
- **Test framework**: cargo test
- **Build command**: `cargo check -p backend`

## Risk Assessment

- [x] Breaking changes to existing code — impact assessed
- [ ] Long-running tests — timeout configured

## Deliverables

- `SchedulingSnapshot` 承载 request record policy 所需字段
- 审计路径改为从 snapshot 取 policy
- 单测覆盖 snapshot-backed policy 恢复

## Done-When

- [ ] policy 不再通过 `SettingStore::get_system_settings()` 逐请求加载
- [ ] snapshot 恢复逻辑测试通过

## Final Validation Command

```bash
cargo test -p backend llm_proxy::request_record_policy -- --nocapture
```

