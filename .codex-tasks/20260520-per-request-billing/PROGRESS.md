# 进展
- 2026-05-20: 已定位根因，`attempt_billing()` 在 `usage.is_none()` 时直接返回，导致按次计费没机会进入统一计费引擎。
- 2026-05-20: 已修复为成功请求统一走计费引擎；无 usage 时使用空 dimensions，request-only billing 也会写入 settled / settlement / usage 记录。
- 2026-05-20: 已验证 `cargo test -p provider billing::service -- --nocapture` 和 `cargo test -p backend success_without_usage_still_settles_request_only_billing -- --nocapture`。
