# Progress

## Recovery

任务: billing/access fixes.
形态: single-full
进度: 6/6
当前: Completed.
文件: `.codex-tasks/20260514-billing-access-fixes/TODO.csv`
下一步: None.

## Log

- 2026-05-14: 初始化修复任务。目标是修复上一轮真实验证暴露的未生效项，并复跑真实 DB/upstream 验证。
- 2026-05-14: 完成只读排查，根因记录在 `raw/implementation-notes.md`。开始实现访问、结算、调度修复。
- 2026-05-14: 完成后端修复：用户禁用、token 额度、钱包余额在调度前拦截；成功 LLM 结算写钱包消费流水和 JSON 快照；token auth cache 保留 used_quota 并在结算后失效；cache_affinity 冷启动无缓存时按请求维度打散同优先级 provider。
- 2026-05-14: 完成真实脚本增强：断言钱包扣费、consume/llm_model_usage 流水、llm_request_record 关联和可审计快照，并为本地旧库补 request snapshot 列检查。
- 2026-05-14: 验证通过：`just check`、`cargo test -q -p proxy scheduler`、`cargo test -q -p backend llm_proxy`、`git diff --check`、4 个 `node --check`、真实 DB/upstream 脚本第二轮 14/14 PASS。
- 2026-05-14: 记录格式检查边界：`cargo fmt --all -- --check` 暴露仓库中 unrelated 旧文件格式差异；未执行全仓格式化，避免改动本任务外文件。
