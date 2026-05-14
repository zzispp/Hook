# Progress

## Recovery

任务: request-record redesign 真实流验证。
形态: single-full
进度: 5/5
当前: Completed.
文件: `.codex-tasks/20260514-real-request-record-flow/TODO.csv`
下一步: None.

## Log

- 2026-05-14: 初始化任务，目标是把调度、降级、格式转换、provider/model 限制、100 并发以及 request-record 新语义一次性做真实验证。
- 2026-05-14: 已确认三家 upstream 的模型探测可用：`gpt-5.5`、`claude-opus-4-7`、`[满血]gemini-3.1-pro-preview`。
- 2026-05-14: 用户已关闭占用中的本地 `5555` 后端，测试将直接接管默认端口并允许中途重启验证 cleanup/sweep。
- 2026-05-14: 已完成新的真实流 harness，按模块拆成 routing 场景、request-record 场景、backend 会话和 request-record 支撑查询，所有脚本 `node --check` 通过。
- 2026-05-14: 第一轮真实跑暴露的都是现场性问题而非静态问题：Claude provider 一度返回 `model_not_found`，endpoint fallback 没有多余 `skipped` 行，压缩详情断言取错了 detail 顶层字段，stale sweep 影子 candidate id 超过 36 位。均已定位并修正 harness 假设。
- 2026-05-14: 第二轮真实跑全部通过，结果写入 `.codex-tasks/20260514-real-request-record-flow/raw/results.json`。
- 2026-05-14: 本轮真实验证覆盖成功：fixed order、cache affinity、load balance、provider failover、endpoint fallback、Claude key failover、OpenAI/Claude/Gemini 非流与流式格式转换、user provider/model allow/deny、100 并发、结构化 `error_code/error_param`、客户端取消 `cancelled/499/client_disconnected`、payload 压缩保留、stale pending/streaming sweep、request record 列表与详情可见性。
- 2026-05-14: 结果库内计数：`request_records` 为 `success=138 failed=3 cancelled=1`；`request_candidates` 为 `success=138 failed=8 skipped=2 cancelled=1`。失败候选来自刻意制造的降级、结构化错误提取和 stale sweep 场景。
- 2026-05-14: 用户后续补充的 `https://www.hook.rs` Claude 备用上游未纳入主流程；单独最小探针在 10 秒窗口内超时，主流程最终仍以成功打通的 AIPAI Claude upstream 为准。
