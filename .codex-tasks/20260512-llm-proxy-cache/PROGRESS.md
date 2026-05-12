# Progress

## Recovery

- 任务: 为 LLM proxy 增加 Redis 鉴权缓存和调度快照缓存。
- 形态: single-full。
- 进度: 5/5。
- 当前: 实现和验证完成。
- 下一步: 无。

## Notes

- 缓存重建必须在 DB CUD 完成后执行；不在业务事务内访问 Redis。
- 请求流水和 token usage 是事实写入，第一阶段不改为 Redis 异步落库。
- 请求路径现在只通过 Redis auth cache 和 Redis scheduling snapshot 获取策略；cache miss 才触发 DB 读取。
- CUD hook 通过 use-case wrapper 执行，inner DB 写返回后再刷新 Redis，避免扩大 DB 锁范围。
- 验证通过：`cargo check -p backend -p proxy`、`cargo test -p proxy --test scheduler`、`cargo test -p proxy --test format_conversion`、`cargo test -p proxy --test request_candidate`、`cargo test -p backend llm_proxy -- --nocapture`。
