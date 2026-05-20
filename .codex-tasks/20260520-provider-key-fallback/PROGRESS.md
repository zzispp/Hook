# Progress

- 2026-05-20: 开始排查 Hook 调度链路。
- 2026-05-20: Hook 根因：`candidate::selection::route` 生成同 provider 多 endpoint/key route，但 `proxy::affinity::effective_max_retries` 和 `ws::connect::attempt_range` 对非缓存候选返回 0，导致只调用 route[0]。
- 2026-05-20: Aether 对齐点：普通 ProviderCandidate 按 key 构建多个候选；PoolCandidate 在候选内部按 `pool_keys` 切换 key。Hook 已采用候选内 route 设计，修复应让非缓存候选至少覆盖全部 route options 一次。
- 2026-05-20: 已修复 HTTP 与 WebSocket 执行层。非缓存候选现在按 route option 数量覆盖 endpoint/key 组合；401/402/403 分类为 `NextCandidate`，命中后跳出当前 provider 候选，不继续该 provider 的其他 key。
- 2026-05-20: WebSocket 握手保留 HTTP status/body，realtime 401/402/403 可走同一 provider 跳过语义；429/5xx/Timeout 仍保留同 provider route 后续 key 的尝试机会。
- 2026-05-20: 验证：`cargo fmt --all -- --check`、`cargo test -p req`、`cargo test -p backend` 通过。`cargo clippy -p backend --all-targets -- -D warnings` 被既有 `crates/storage/src/operations/notification_query.rs:63` 的 `clippy::unnecessary-sort-by` 阻塞。全 workspace `cargo test` 在 60 秒上限超时，已完成部分均通过。
