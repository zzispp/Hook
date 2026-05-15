# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- 将 `llm_proxy` 成功请求后的 token usage / model usage 同步 DB 写，从客户请求主链路移到 Redis 聚合 + 后台 flush。
- 保持 quota runtime 判定继续由 Redis/cache 驱动，不引入静默丢数。

## Non-Goals

- 不在本任务内异步化钱包结算。
- 不在本任务内异步化 `request_records` / `request_candidates` 历史审计写入。

## Constraints

- Redis 写失败必须显式暴露，不能静默跳过。
- flush 失败时 pending 数据必须可重试，不能丢失。
- 代码改动继续遵守文件大小和函数长度限制。

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024
- **Package manager**: cargo / just
- **Test framework**: cargo test

## Deliverables

- `LlmProxyCache` 支持累积 token/model usage pending state
- 后台 flush worker 批量落库 token/model usage
- 热路径移除 token/model usage 的同步 DB 写

## Done-When

- [ ] 成功请求主链路不再直接调用 token/model usage DB `record_usage`
- [ ] flush worker 失败时 pending 数据仍保留可重试
- [ ] 后端检查与测试通过

## Final Validation Command

```bash
just test
```
