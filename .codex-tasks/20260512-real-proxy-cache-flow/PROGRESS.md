# Progress

## 2026-05-12

- Created the task to validate the real proxy flow after the Redis cache module was added.
- Confirmed the local backend was not listening on `127.0.0.1:5555` before the test run.
- Confirmed the prior reference script already seeds deterministic provider/key/model fixtures and exercises core scheduling and conversion paths.
- Added `.codex-tasks/20260512-real-proxy-cache-flow/real_proxy_cache_flow.mjs` and helper modules. All secrets are read from environment variables.
- `cargo check -p backend -p proxy` passed.

## Real Run Summary

Command shape:

```sh
HOOK_SYSTEM_TOKEN=... HOOK_POOL_BASE_URL=https://www.hook.rs HOOK_POOL_KEY=... CLAUDE_KEY=... CLAUDE_BASE_URL=https://www.hook.rs EKAN8_KEY=... node .codex-tasks/20260512-real-proxy-cache-flow/real_proxy_cache_flow.mjs
```

The script starts `cargo run -p backend` when `127.0.0.1:5555` is not already healthy, seeds deterministic DB fixtures, clears Redis proxy caches, runs each scenario, and kills the backend it started.

Passed real scenarios:

- Settings API cache hook: `PATCH /api/admin/settings/system` refreshed `hook:llm_proxy:scheduling:snapshot:v1`.
- Auth cache hook: create/delete admin token bumped `hook:llm_proxy:auth:version`.
- FixedOrder non-stream OpenAI exact: success, `openai_chat -> openai_chat`, no conversion.
- FixedOrder stream OpenAI exact: success, `is_stream=true`, `first_byte_time_ms` and final `latency_ms` recorded.
- Failover: broken provider/key returned two 401 `upstream_status` rows, then transferred to the real Hook OpenAI provider and succeeded.
- CacheAffinity: preseeded Redis affinity key promoted `Hook secondary`.
- LoadBalance: real calls distributed across `Hook primary` and `Hook secondary`.
- OpenAI-to-Gemini conversion: success, `openai_chat -> gemini_chat`, conversion marked true.
- Claude-to-OpenAI conversion: success, `claude_chat -> openai_chat`, conversion marked true.
- Gemini exact non-stream: success, `gemini_chat -> gemini_chat`, no conversion.
- Gemini exact stream: success, `is_stream=true`, first-byte and final latency recorded.
- Concurrent rebuild stress: 8 non-stream + 3 stream requests while 6 API scheduling-mode PATCH operations rebuilt the scheduling snapshot; all requests succeeded, both keys were used, and the Redis rebuild lock was released. A follow-up narrow run with 2 + 1 requests also passed.

Blocked real scenario:

- OpenAI-to-Claude conversion reached the Claude provider candidate and recorded two `upstream_timeout` retries, then returned 502. Direct probes showed `https://www.hook.rs/v1/messages` did not respond within 15 seconds for all tested Claude models and previously returned Cloudflare 524 after about 125 seconds. `https://pool.hook.rs/v1/messages` returned 502 for Claude. `https://api.aipaibox.com` failed TLS handshake from this machine. This is a real upstream availability problem, not a local mock or silent fallback.

Recent DB evidence:

- `019e1bbb-7bd8-7fb3-9ef6-eda5b85f7cb7`: FixedOrder OpenAI exact success.
- `019e1bbb-8501-74c1-9bc6-381d33d30114`: FixedOrder OpenAI stream success.
- `019e1bbb-a103-7622-932d-12ad56bc368f`: broken provider 401 retries, then failover success.
- `019e1bbb-aec3-72e3-bb8e-d4187d96006b`: CacheAffinity success on `Hook secondary`.
- `019e1bbc-cc9a-7350-8fb9-f0837d70143c`: OpenAI-to-Gemini conversion success.
- `019e1bbc-f7f5-71a2-b5b5-70df390f7368`: Claude-to-OpenAI conversion success.
- `019e1bbd-0e59-7113-afe9-665f36f204b2`: Gemini exact non-stream success.
- `019e1bbd-3cf1-78a1-abee-4a3998ce705d`: Gemini exact stream success.
- `019e1bbd-56fd-7b60-b7b9-a4d9eec5f9ad`, `019e1bbd-56fd-7b60-b7b9-a4ec3ca47141`, `019e1bbd-573d-78d0-b268-f53fa0ad0cf9`: narrow concurrent rebuild stress success.
- `019e1bbc-2fa2-7ef1-bbca-c5b93648ff7f`: OpenAI-to-Claude blocked by `upstream_timeout`.

Additional DB check:

- All request rows for the seeded independent token were owned by admin user `00000000-0000-7000-8000-000000000000`.
