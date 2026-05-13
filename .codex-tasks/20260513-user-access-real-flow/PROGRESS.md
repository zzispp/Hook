# Progress

## Recovery

任务: Real end-to-end validation for user allowed provider/model restrictions.
形态: single-full
进度: 5/5
当前: Completed.
文件: `.codex-tasks/20260513-user-access-real-flow/TODO.csv`
下一步: None.

## Log

- 2026-05-13: 初始化任务。
- 2026-05-13: 已读取 `.codex-tasks/20260513-real-route-scheduler-flow`，旧真实脚本已覆盖 route scheduler、降级、格式转换、流式/非流式和 32 并发。
- 2026-05-13: 本地数据库当前缺少 `users.allowed_model_ids` 与 `users.allowed_provider_ids`；真实测试脚本会显式做非破坏性 `ADD COLUMN IF NOT EXISTS`，避免重置数据库。
- 2026-05-13: 已完成 user access 真实测试 harness，覆盖用户级 allowed provider/model、编辑用户 API 刷新调度缓存、fixed order、cache affinity、load balance、key failover、endpoint fallback、provider failover、格式转换矩阵。
- 2026-05-13: 已修复启动阶段 scheduling snapshot 使用 `u64::MAX` 作为分页 limit 导致 Postgres/SQLx 整数转换 panic 的问题，改为 `i64::MAX as u64`。
- 2026-05-13: 已通过静态与自动化验证：`cargo fmt --check`、`cargo check -p backend`、`cargo test -p backend`、`cargo test -p proxy`、`pnpm lint:frontend`、`pnpm build:frontend`。
- 2026-05-13: 真实流程测试已通过，结果写入 `.codex-tasks/20260513-user-access-real-flow/raw/results.json`。
- 2026-05-13: 按用户要求清空 `request_candidates` 后重新真实测试，全部场景再次通过；本轮生成 145 条请求候选记录，其中 success 非流 66、success 流式 74、failed 非流 5，active pending/streaming 为 0。5 条 failed 均为测试刻意制造的降级尝试。

## Latest Real Test

- 模型：`gpt-5.5`、`claude-opus-4-7`、`gemini-3.1-pro-preview`。
- Provider：Hook Pool OpenAI、AI 派 Claude、Ekan8 Gemini。
- 用户限制：unrestricted、OpenAI-only、Claude-only、Gemini-only、model-only、provider mismatch、API edit 后限制刷新。
- 调度：fixed order、cache affinity、load balance。
- 降级：Claude key failover、OpenAI endpoint fallback 到 Responses 格式转换、broken provider failover。
- 格式转换：OpenAI->Claude、OpenAI stream->Claude、OpenAI->Gemini、OpenAI stream->Gemini、Claude->OpenAI、Gemini exact、Gemini stream exact。
- 高并发：100 并发，30 非流 + 70 流式，Hook Pool primary/secondary 均被使用。
