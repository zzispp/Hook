# Progress

## Recovery

任务: 用户 / 令牌 / Provider Key 速率限制运行时修复与真实验证。
形态: single-full
进度: 6/6
当前: 全部完成。
文件: `.codex-tasks/20260514-rate-limit-real-flow/TODO.csv`
下一步: 无。

## Log

- 2026-05-14: 初始化任务。
- 2026-05-14: 已完成运行时限流实现。用户限制在请求入口执行；令牌限制在请求入口执行并遵循 `token/system` 更严格者；Provider Key 限制在每次上游 attempt 前执行。
- 2026-05-14: 已完成 Provider Key 表单文案调整，标签改为“速率限制(请求/分钟)”，helper 改为“0 表示不限制”，默认值为 `0`。
- 2026-05-14: 初次真实验证中，用户限流和 Provider Key 全耗尽两个 `1 RPM` 场景偶发失败；trace 与已通过场景一致，判断为固定分钟桶跨分钟导致的脚本漂移，不是运行时主链缺失。
- 2026-05-14 16:07:43 +0800: 再次核对当前执行点，确认 `apps/hook_backend/src/llm_proxy/proxy/request.rs` 与 `apps/hook_backend/src/llm_proxy/ws.rs` 调用 `enforce_request_limits`，`apps/hook_backend/src/llm_proxy/proxy/executor.rs` 与 `apps/hook_backend/src/llm_proxy/ws/connect.rs` 调用 `claim_provider_key_limit`。
- 2026-05-14 16:07:43 +0800: 已在 `.codex-tasks/20260514-rate-limit-real-flow/rate_limit_real_flow.mjs` 增加分钟窗口对齐逻辑，确保所有 `1 RPM` 场景仅在剩余窗口足够时开始。
- 2026-05-14 16:12:49 +0800: 使用新的 Hook Pool 配置 `https://www.hook.rs` + 新 key 完成真实请求验证，`raw/results.json` 全部场景为 `ok: true`。
- 2026-05-14 16:12:49 +0800: 真实验证结果确认如下。
  - 用户限制、令牌不限: 第 1 次 200，第 2 次 429，错误为 `user rate limit exceeded`。
  - 令牌限制、用户不限: 第 1 次 200，第 2 次 429，错误为 `token rate limit exceeded`。
  - 用户限制叠加令牌限制: 第 1 次 200，第 2 次 429，命中更严格的用户限制。
  - 单用户多令牌: token A 成功后，token B 立即 429，说明共享用户桶生效。
  - Provider Key 单 key 限流: primary 第一次成功，第二次被 `provider_key_rate_limit` 拦下并切到 secondary 成功。
  - Provider Key 双 key 全耗尽: 第三次请求返回 429，所有 candidate trace 都是 `provider_key_rate_limit`。
- 2026-05-14 16:12:49 +0800: 静态检查完成，`cargo check -p backend` 与 `pnpm lint:frontend` 均通过。
