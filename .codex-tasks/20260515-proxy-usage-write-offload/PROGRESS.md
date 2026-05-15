# Progress Log

## Context Recovery Block

- **Current milestone**: complete
- **Current status**: DONE
- **Last completed**: #3 — Switch hot path and validate storage/tests
- **Current artifact**: `TODO.csv`
- **Key context**: 成功请求主链路里的 token usage / model usage 同步 DB 写已移到 Redis pending 聚合 + 后台 flush，不碰钱包结算和 request audit 历史写。
- **Known issues**: token/model usage 落库失败会暴露为后台错误日志，并把 processing 数据回灌 pending 重试；不会静默丢数。
- **Next action**: none

## 2026-05-15

- 设计并实现 pending/processing Redis hash：token cost/count/last_used_at 与 model count 分开聚合。
- token cost 采用 8 位定点整数写 Redis，避免 `used_quota` 经 Redis 浮点累计产生精度漂移。
- flush worker 使用 Redis `SET NX EX` 锁；启动每轮先恢复 leftover processing，再原子移动 pending 到 processing。
- storage 层新增 token/model usage batch API，每类 batch 用一个 DB transaction，避免半批成功后整批回灌造成重复计数。
- `audit.rs` 成功请求路径保留 runtime auth cache 更新，并改为 enqueue token/model usage pending，不再同步调用 token/model usage DB `record_usage`。
- 验证通过：`just format`、`cargo check -p backend`、`cargo clippy -p backend --all-targets -- -D warnings`、`just check`、`cargo test -p backend usage_flush -- --nocapture`、`just test`。
