# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- 停止“每次成功请求后 bump 全局 auth version”这一行为。
- 保持 cached token 的 `used_quota/request_count/last_used_at` 与 DB 记录一致，不依赖下一次回源 DB。

## Non-Goals

- 不在本子任务内引入异步 token usage flush。

## Constraints

- 不改变现有 quota 判定语义。
- 不为 Redis/DB 失败引入静默降级。

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

- `llm_proxy` auth cache 支持基于 token id 的缓存 usage 同步
- 成功计费后不再 bump 全局 auth version
- 单测覆盖缓存 usage 更新行为

## Done-When

- [ ] 认证缓存用量同步逻辑通过测试
- [ ] 成功 usage 记录路径不再 bump 全局 auth version

## Final Validation Command

```bash
cargo test -p backend llm_proxy::cache::auth -- --nocapture
```

