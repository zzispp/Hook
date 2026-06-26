# Progress

- 任务创建。
- 已确认根因：
  - `crates/provider/src/application/service/quick_import_sync.rs` 的 `run_quick_import_sync` 外层按 source 顺序执行，任何未处理的 `Err` 都会中断整批。
  - `sync_source` 里 `fetch_sync_snapshot` 失败会进入 `handle_source_failure(...)`，但 `refreshed_source_config(...)` 的 401 刷新失败原先直接冒泡，导致 source 失败既没有被记账，也会阻塞后续 source。
- 已完成修复：
  - 将 `refreshed_source_config(...)` 的错误改为与 `fetch_sync_snapshot(...)` 同路径处理，统一走 `handle_source_failure(...)`，这样单个 provider 登录信息失效时会记录失败并继续处理下一项。
  - 保留 `run_quick_import_sync` 外层对未处理基础设施错误的显式失败语义，没有引入静默降级或吞错逻辑。
- 已补回归测试：
  - `crates/provider/src/application/service/quick_import_sync_tests.rs`
  - `crates/provider/src/application/service/quick_import_sync_test_support.rs`
  - 覆盖场景：第一个 source 在 `refreshed_source_config` 阶段返回 sub2api 401 后，`report.failed_count == 1`、后续第二个 source 仍继续同步成功、并产生对应 source failure event。
- 验证结果：
  - `cargo fmt --all`：通过。
  - `cargo test -p provider quick_import_sync -- --nocapture`：通过。
  - `cargo test -p provider -- --nocapture`：通过。
  - `cargo test -p hook_backend scheduled_tasks::tests::provider_quick_import_sync -- --nocapture`：通过。
  - `cargo clippy -p hook_backend --all-targets -- -D warnings`：通过。
  - `just test`：未通过，原因不是断言失败，而是 `justfile` 第 1 行的 `test_timeout_seconds := "60"` 与第 19 行的 Perl 超时包装器在 60 秒处终止 `cargo test`，最终退出码 `124`。
