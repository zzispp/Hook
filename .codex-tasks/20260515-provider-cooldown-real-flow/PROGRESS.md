# Progress

2026-05-15
- Started real provider cooldown flow based on existing real request-record and usage concurrency harnesses.
- Local DB inspection showed current database lacks `provider_cooldowns` and `system_settings.provider_cooldown_policy`; the script will explicitly prepare those schema objects before starting the backend.
- Built `.codex-tasks/20260515-provider-cooldown-real-flow/provider_cooldown_real_flow.mjs` and split helpers under 300 lines. Static validation passed with `node --check` for the main script and all helper modules.
- First real run exposed a backend Redis pipeline bug in provider cooldown recording: `record_failure_event` decoded a Redis pipeline response as `i64`, while the actual response was a one-item array. The client received `502 infrastructure_error`, and no DB cooldown row was written.
- Fixed `apps/hook_backend/src/llm_proxy/cache/provider_cooldown.rs` to decode the Redis pipeline as `(i64,)`. `perl -e 'alarm shift; exec @ARGV' 60 cargo check -p backend` passed.
- Final real run passed with temporary exported upstream keys only:
  - Policy: status `404`, threshold `1`, window `60s`, cooldown `120s`.
  - Trigger request `019e2b78-1219-70d3-833e-ee24fd6b35a2`: Msutools candidate failed with upstream `404`; cooldown DB row stored request/candidate/error details; Redis cooldown key value was `1`; failure ZSET count was `1`.
  - Follow-up request `019e2b78-141a-7c30-8d7a-fd023966de5b`: cooled Msutools provider was absent from candidates; Ekan8 succeeded with `openai_chat -> gemini_chat` conversion and status `200`.
  - Manual release API returned the Msutools provider id and left cooldown list total `0`; script confirmed active DB cooldown query returned no row and Redis cooldown key was deleted.
- Cleanup restored `system_settings.provider_cooldown_policy` to `{"window_seconds":0,"rules":[]}` and removed the test providers from the local DB. Final evidence is in `raw/results.json`; key grep found no raw upstream secrets in the task files.
