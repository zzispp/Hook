# Progress

## Recovery

任务: 真实验证模型调用后 `global_models.usage_count` 是否递增。
形态: single-full
进度: 5/5
当前: 已完成。
文件: `.codex-tasks/20260514-model-usage-real-flow/TODO.csv`
下一步: None.

## Log

- 2026-05-14: 用户要求补一套更聚焦的真实测试，目标不是 request-record 全矩阵，而是直接验证模型管理里的 `usage_count` 在真实调用后会增加。
- 2026-05-14: 已复盘 `.codex-tasks/20260514-real-request-record-flow/` 的现成做法，确认可以复用 backend 启动、代理请求、Redis 清缓存、request record 查询等现有 helper。
- 2026-05-14: 已确认本地依赖现状：
  - Postgres 由 Docker 容器 `hook-postgres` 暴露在 `localhost:5433`。
  - Redis 由 Docker 容器 `hook-redis` 暴露在 `localhost:6380`。
  - 当前 `127.0.0.1:5555` 没有 backend 监听，脚本可自行接管。
- 2026-05-14: 已现场探测真实上游：
  - `https://www.hook.rs/v1/models` 当前可返回 `gpt-5.5`、`gpt-5.4`、`claude-opus-4-7` 等模型。
  - `https://www.ekan8.com/v1/models` 与 `https://www.ekan8.com/v1beta/models` 当前 body 中可返回可用模型列表，包含 OpenAI 兼容可见的 `[满血]gemini-3.1-pro-preview` 等 alias；后续脚本会再做完整解析与可用性探针。
- 2026-05-14: 已新增任务脚本：
  - `real_model_usage_count_flow.mjs`：真实探测上游模型、写本地 DB 夹具、起本地 backend、发真实请求并查询 `usage_count` 前后差值。
  - `lib/docker_db.mjs`：通过 `docker exec hook-postgres psql` 直接连接本地容器数据库，不依赖宿主机 PATH 中有 `psql`。
- 2026-05-14: 本次真实脚本执行使用了两条链路：
  - 直连链路：客户端模型 `usage-real-openai-direct` -> Hook.rs 上游模型 `gpt-5.5`
  - 映射链路：客户端模型 `usage-real-openai-mapped` -> Ekan8 映射 alias `R-claude-opus-4-7` -> 上游响应模型 `claude-opus-4-7`
- 2026-05-14: 脚本实际结果：
  - 本地 backend 在 `5555` 上由脚本启动并成功处理两条真实请求。
  - request id:
    - direct: `019e2617-0365-7793-989d-9b297a95aa7f`
    - mapped: `019e2617-13b6-7bc0-8432-2168c35bb9ad`
  - `global_models.usage_count`：
    - `usage-real-openai-direct`: `0 -> 1`
    - `usage-real-openai-mapped`: `0 -> 1`
  - `api_tokens.request_count`: `0 -> 2`
  - `request_records` 两条都为 `success / settled`。
- 2026-05-14: 裸 SQL 复核通过：
  - `global_models` 查询确认两个测试模型当前 `usage_count = 1` 且 `is_active = true`。
  - `request_candidates` 查询确认：
    - direct 成功候选 `provider_request_body.model = gpt-5.5`
    - mapped 成功候选 `provider_request_body.model = R-claude-opus-4-7`
    - mapped 成功候选 `provider_response_body.model = claude-opus-4-7`
  - 说明本次不仅计数增加，而且映射命中与响应 model 回写链路也真实发生了。
