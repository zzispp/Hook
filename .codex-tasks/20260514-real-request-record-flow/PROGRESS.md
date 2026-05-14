# Progress

## Recovery

任务: request-record redesign 真实流验证。
形态: single-full
进度: 8/8
当前: Completed with upstream blockers documented.
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
- 2026-05-14: 用户补充的新一轮要求是基于当前“模型映射 + reasoning_effort + 响应 model 回写”改动，复用现有 harness 再做一轮真实回归，不另起第二套夹具。
- 2026-05-14: 已现场确认真实上游能力：`https://pool.hook.rs/v1/models` 可见 `gpt-5.4` / `gpt-5.4-mini` / `gpt-5.5`；`https://api.aipaibox.com/v1/models` 可见 `claude-opus-4-7`；`https://www.ekan8.com` 同时支持 OpenAI 兼容 `/v1/models` 与 Gemini `/v1beta/models`，且 `R-claude-opus-4-7`、`ccmax-claude-opus-4-7` 等别名都能真实完成请求。
- 2026-05-14: 计划把现有 OpenAI route fixture 在新场景中临时切到 Ekan8 OpenAI 兼容端点，通过 admin API 修改 endpoint/key/model binding，直接验证上游模型拉取、1:1 映射命中、`reasoning_effort` 注入、响应 `model` 回写以及编辑后缓存重建是否立即生效。
- 2026-05-14: 已完成 harness 扩展：
  - 新增 `request_record_real_provider_admin.mjs`，通过 admin API 真实更新 provider endpoint、key、model binding 并拉取 upstream models。
  - 新增 `request_record_real_mapping_scenarios.mjs`，覆盖上游模型拉取、Ekan8 OpenAI 兼容映射、映射级 `reasoning_effort`、响应 `model` 回写和编辑后缓存重建。
- 2026-05-14: 真实映射链路已被命中并验证通过：
  - admin `upstream-models` 成功返回 `Hook Pool -> [gpt-5.5, gpt-5.4-mini]`、`Claude fixture -> [R-claude-opus-4-7, ccmax-claude-opus-4-7]`、`Ekan8 Gemini -> [gemini-3.1-pro-preview, [满血]gemini-3.1-pro-preview]`。
  - 映射非流请求 `019e252e-1a54-7111-8a59-3a2fcfe078c9` 真实把客户端 `gpt-5.5` 路由到上游 `R-claude-opus-4-7`，请求体记录了 `reasoning_effort=high`，客户端响应 `model` 被回写成 `gpt-5.5`。
  - 映射流式请求 `019e252e-559a-7471-882b-e58bb27a031b` 真实把客户端 `gpt-5.5` 路由到上游 `ccmax-claude-opus-4-7`，请求体记录了 `reasoning_effort=minimal`，所有流式 `model` 字段都被回写成客户端模型，没有泄露上游 alias。
  - 两次请求之间仅通过 admin API 修改 binding，没有手工清理 scheduling cache；第二次请求立即命中新映射，证明缓存重建链路生效。
- 2026-05-14: 多轮真实复跑显示，上游现场在本次窗口内持续漂移，导致“整套历史矩阵全绿”目前不可稳定复现：
  - AIPAI Claude：
    - 早先一轮真实跑曾成功完成 OpenAI -> Claude 非流/流式转换。
    - 后续同日复跑出现 `AbortError`、`429`，以及当前直接探针 `/v1/messages` 返回 `model_not_found`：`No available channel for model claude-opus-4-7 under group CC 自营 满血反重力 (distributor)`。
  - Hook Pool OpenAI：
    - 单独直接探针 `gpt-5.5` 仍可返回 200。
    - 但在矩阵复跑中，`openai_chat` 直通多次失败后被迫 fallback 到 `openai_cli` / `openai_compact`；其中 `openai_compact` 现返回 `Unsupported parameter: max_output_tokens`，直接击穿了 endpoint fallback 和结构化错误断言。
    - 在 housekeeping 与 `100 concurrent mixed requests` 场景里，多次出现 undici `TypeError: terminated`，继而连带 `request record list and detail visibility` 断言失败。
- 2026-05-14: 因此，本轮结论不是“代码回归失败”，而是：
  - 新增模型映射能力本身已经通过真实请求验证。
  - 获取上游模型的管理端链路已经通过真实请求验证。
  - 旧有大矩阵在今天的真实上游环境下受到 AIPAI 与 Hook Pool 两侧漂移阻塞，失败点已全部显式记录在 `raw/results.json`，未做任何静默 fallback。
