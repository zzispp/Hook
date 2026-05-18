# LLM Proxy Format Routing

## Recovery

- 任务: 对齐模型测试与主 LLM Proxy 的跨格式候选路由
- 形态: single-full
- 进度: 4/4
- 当前: 已完成 Hook 模型测试路由对齐，并用测试覆盖主代理 Gemini/Claude -> OpenAI endpoint/key 候选选择
- 文件: `.codex-tasks/20260518-llm-proxy-format-routing/TODO.csv`
- 下一步: 汇总变更和验证结果

## Notes

- aether 主代理通过 CandidateBuilder 按客户端格式构建候选，key 按实际 provider endpoint format 过滤。
- aether 模型测试接口传入 `endpoint_id` 时会固定 endpoint，但其 candidate builder 本身仍支持 client/provider format 分离。
- Hook 主代理 `matching.rs` 已按 `request.api_format` 与 endpoint format 分开匹配，偏差集中在 `model_test`。
- Hook 模型测试现在把选中的 endpoint 视为测试入口格式，实际 provider endpoint/key 由兼容候选选择出来。
