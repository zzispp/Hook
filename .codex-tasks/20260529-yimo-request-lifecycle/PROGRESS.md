# 进度记录

## 2026-05-29

- 已确认线上现象并抓取证据：`yimo` 请求会悬挂在 `pending`，只能被 `request_record_stale_sweep` 定时回收。
- 正在并行分析 Aether 与 Hook 的请求生命周期实现，聚焦发送超时、首字节超时、流式 EOF、取消和候选切换路径。
- 已完成 Aether 对照结论：Aether 的关键不是 sweep，而是“整个候选执行链自己把 pending 收到终态”，其中 sync 侧靠 terminal guard，stream 侧靠 watchdog + handoff。
- 已在 Hook 中把 `AttemptCancelGuard` 延长到真正终态或 `StreamRelay` 接手，不再在拿到上游响应头后立刻撤防；同时补上“响应已开始但未终态时被取消”的单独落库语义。
- 已移除 `request_record_stale_sweep` 定时任务、storage sweep 模块和对应的任务文案；保留 `stale_*` trace label 文案以兼容历史记录展示。
- 已补充测试并验证：
  - `cargo test -p backend llm_proxy::proxy::attempt_log::tests -- --nocapture`
  - `cargo test -p scheduler list_tasks_ignores_unregistered_database_rows -- --nocapture`
  - `cargo clippy -p backend --all-targets --no-deps -- -D warnings`
- 全量验证状态：
  - `just test` 运行通过大部分用例，但被仓库既有失败 `llm_proxy::formats::tests::streaming_requests_do_not_route_to_force_non_stream_formats` 拦住。
  - `cargo clippy -p backend --all-targets -- -D warnings` 仍被仓库既有 `crates/formats` / `crates/types` 历史告警拦住，不是本次改动引入。
- 用户追加要求：继续按 Aether 的思路补一层更外层的 stream candidate watchdog，把整条 stream candidate 执行包进 `tokio::spawn + timeout`，超时后 `abort`，显式落失败终态，再继续后备候选。
- 当前 watchdog 骨架已接入 `executor.rs`，并新增 `stream_candidate_watchdog_timeout = first_byte_timeout + 1s handoff slack`，目的是避免与现有 `prefetch_with_timeout` 在同一刻竞争终态。
- 待收尾项：
  - 去掉 `executor.rs` 里为复用非 2xx 失败处理而临时伪造 `PreparedProxyRequest` 的逻辑。
  - 统一 `AttemptCancelGuard` 在 stream 路径里的共享引用语义，消除 `&mut` / `&` 混用。
  - 补上 watchdog timeout helper 测试与记录语义测试，再跑后端验证命令。
- 本轮收尾结果：
  - 已将 stream candidate 执行包进外层 `tokio::spawn + timeout` watchdog；超时会先 `disarm` guard，再 `abort` 任务，显式记录 `local_stream_candidate_watchdog_timeout` 为 `failed`，并返回 `NextCandidate`。
  - 已去掉 `executor.rs` 中为复用非 2xx 失败路径而临时伪造 `PreparedProxyRequest` 的做法，改成只依赖 `request_id` 的失败处理参数。
  - 已把 stream handoff 路径的 `AttemptCancelGuard` 引用统一成共享只读引用，避免 stream watchdog 分支与 relay handoff 分支继续混用 `&mut`。
- 最新验证：
  - 通过：`cargo fmt --all`
  - 通过：`cargo check -p backend`
  - 通过：`cargo test -p backend llm_proxy::proxy::attempt_log::tests -- --nocapture`
  - 通过：`cargo test -p backend llm_proxy::proxy::executor::tests -- --nocapture`
  - 通过：`cargo test -p backend llm_proxy::proxy::timeout::tests -- --nocapture`
  - 通过：`cargo clippy -p backend --all-targets --no-deps -- -D warnings`
  - 失败但与本次改动无关：`just test` 仍被既有失败 `llm_proxy::formats::tests::streaming_requests_do_not_route_to_force_non_stream_formats` 拦住。
  - 失败但与本次改动无关：`cargo clippy -p backend --all-targets -- -D warnings` 仍被既有 `crates/formats` 大量 `collapsible_if` 告警和 `crates/types` 的 `large_enum_variant` 告警拦住。
