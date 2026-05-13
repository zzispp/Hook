# Progress

## Recovery

任务: Real end-to-end route scheduler validation after candidate route refactor.
形态: single-full
进度: 6/6
当前: Completed.
文件: .codex-tasks/20260513-real-route-scheduler-flow/TODO.csv
下一步: None.

## Log

- Read the previous real proxy cache flow under `.codex-tasks/20260512-real-proxy-cache-flow`.
- Probed the supplied real upstreams:
  - Hook Pool OpenAI-compatible `/v1/models` returned available OpenAI models including `gpt-5.5`.
  - AIPAI Claude-compatible `/v1/models` returned Claude models including `claude-sonnet-4-6`.
  - Ekan8 Gemini-compatible `/v1beta/models` returned Gemini models including `[满血]gemini-3.1-pro-preview`.
- Created real route scheduler harness:
  - `.codex-tasks/20260513-real-route-scheduler-flow/real_route_scheduler_flow.mjs`
  - `.codex-tasks/20260513-real-route-scheduler-flow/lib/route_fixtures.mjs`
  - `.codex-tasks/20260513-real-route-scheduler-flow/lib/route_client.mjs`
- `node --check` passed for all three harness files.
- Static checks passed:
  - `just test`
  - `pnpm --filter hook_frontend lint`
- First real run exposed two test-harness issues and one important cache-key mismatch:
  - The old reference script cleared `hook:llm_proxy:scheduling:snapshot:v1`, while the current backend uses `snapshot:v2`.
  - `/v1/responses/compact` requires list-shaped `input` and rejects `max_output_tokens`.
  - Hook Pool accepts the invalid-looking OpenAI key used by the first key-failover draft, so key failover was moved to AIPAI Claude where upstream key validation is real.
- Final real run passed all scenarios:
  - fixed order exact OpenAI Chat, OpenAI stream, OpenAI Responses, OpenAI Compact
  - Claude key failover: `019e1f3c-c61c-71e3-a415-5f11c1bd62a8` recorded failed `0.0` then success `0.1`
  - endpoint fallback conversion: `019e1f3c-e182-7960-aa2b-78433f217a51` recorded failed exact key attempts then success on `openai_cli`
  - provider failover: `019e1f3c-ef01-7be2-a131-6d8c959d45d7` recorded broken provider failures then Hook Pool success
  - cache affinity: `019e1f3d-0b7e-74c3-961e-2477d27fb483` used `Route Hook secondary`
  - load balance used both `Route Hook primary` and `Route Hook secondary`
  - format conversion passed OpenAI->Claude, OpenAI stream->Claude, OpenAI->Gemini, OpenAI stream->Gemini, Claude->OpenAI, Gemini exact, Gemini stream exact
  - high concurrency passed 32 concurrent requests: 24 non-stream and 8 stream, with both Hook keys used
- DB evidence query showed zero lingering `available` rows for sampled route/failover/conversion requests.
- Cleanup verification:
  - `system_settings.scheduling_mode` restored to `cache_affinity`
  - route test providers, models, and token are inactive
  - no Redis `*llm_proxy:scheduling*` keys remained
- Raw structured evidence is in `.codex-tasks/20260513-real-route-scheduler-flow/raw/results.json`.
