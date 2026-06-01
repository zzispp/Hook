# Progress

## Recovery

任务: 为模型状态探测增加 provider key 维度的跨实例 Redis 节流
形态: single-full
进度: 4/4
当前: Done
文件: .codex-tasks/20260601-model-status-probe-throttle/TODO.csv
下一步: None.

## Evidence

- Dispatch defaults: batch size 20, concurrency 4.
- Due checks execute through `buffer_unordered(concurrency)`.
- Existing provider-key RPM limiter uses minute buckets and cannot smooth same-second bursts.
- Scheduler running locks are process-local.
- Probe requests already flow through `proxy_json`, so the correct provider-key throttle point is immediately before upstream execution.

## Completed

- Added explicit domain `Deferred` probe result and dispatch report.
- Deferred checks clear their database lock and receive a later `next_due_at` without inserting a run.
- Validation: `timeout 60 cargo test -p model_status run_due_checks_defers_throttled_probe -- --nocapture`.
- Added Redis `SET NX EX` provider-key probe slot before provider-key RPM accounting.
- Added explicit deferred audit closure: scheduled candidates become `skipped`, request summary becomes `skipped` with `void` billing.
- Added scheduled task config `provider_key_min_interval_seconds`; task output now reports `probed_count` and `deferred_count`.
- Validation passed:
  - `cargo check -p backend -p model_status -p storage`
  - `timeout 60 cargo test -p model_status`
  - `timeout 60 cargo test -p backend model_status_probe -- --nocapture`
  - `timeout 60 cargo test -p backend provider_key_probe_throttle -- --nocapture`
  - `timeout 60 cargo test -p backend skipped_request_has_void_billing_status -- --nocapture`
  - `timeout 60 cargo test -p storage due_checks_sql_uses_skip_locked_claiming -- --nocapture`
  - `rustfmt --edition 2024 --check` on touched Rust files.

## Notes

- `cargo fmt --all --check` is currently blocked by unrelated provider/model-binding formatting diffs outside this task.
