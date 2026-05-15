# Progress

## 2026-05-15

- Read the existing `.codex-tasks/20260514-real-request-record-flow` harness and reused its backend, DB, Redis, request, and admin API conventions.
- Confirmed the script requires runtime env variables and does not persist provider secrets in source, SQL artifacts, or result evidence.
- Started implementing a real multi-account, multi-key, high-concurrency flow plus Redis usage recovery checks.
- Implemented the runnable harness:
  - 3 wallet-backed fixture users and 3 customer tokens created through the admin API.
  - 2 providers, 2 provider key records per provider, and load-balance scheduling.
  - Runtime upstream model fetch for provider1 OpenAI-compatible `/v1/models` and Ekan8 Gemini `/v1beta/models`.
  - 36 default concurrent `/v1/chat/completions` calls across all customer tokens.
  - Hard assertions for at least 3 customer tokens, 4 provider key records, and 2 successful providers.
  - DB checks for `request_records`, `request_candidates`, `wallet_transactions`, `api_tokens`, `global_models`.
  - Redis checks for usage pending/processing keys and cold-start processing-batch recovery.
- Added `HOOK_REAL_PROVIDER1_KEYS` and `HOOK_REAL_PROVIDER2_KEYS` comma-separated multi-key env support. Result artifacts store counts and sanitized labels only; raw key values are not returned in evidence or error strings.
- Split cleanup from fixture seeding so all new script files stay under 300 lines; the simple function-size check reports no function over 50 nonblank lines.
- Validation passed: `find .codex-tasks/20260515-real-usage-concurrency-flow -maxdepth 2 -name '*.mjs' -print0 | xargs -0 -n1 node --check`.
- First real execution without provider env stopped at `load runtime context` with `missing required env: HOOK_REAL_PROVIDER1_KEY or HOOK_REAL_PROVIDER1_KEYS`; this happened before DB, Redis, backend startup, or upstream calls.
- A 60-concurrent run against the original provider1 endpoint exposed provider-side/capacity instability: with 180s timeout it aborted a batch; with 300s timeout the traffic finished but the strict successful-key coverage assertion saw only 3 successful key records. Evidence: `raw/results-60-timeout.json` and `raw/results-60-300-keycoverage-fail-hook.json`.
- Provider1 was switched to `https://www.hook.rs`; `HOOK_REAL_PROVIDER1_BASE` now defaults to that endpoint so the harness does not hit `www.hook.rs` unless explicitly configured.
- Real validation passed with provider1 `msutools` and provider2 Ekan8:
  - `raw/results-24-pass.json`: 24 concurrent requests, all `request_records` success/settled.
  - `raw/results-36-pass.json`: 36 concurrent requests, all success/settled; 3 customer tokens each counted 12 requests and 0.00120000 usage; wallet transactions matched 36.
  - `raw/results-60-msutools-pass.json`: 60 concurrent requests, all success/settled; 3 customer tokens each counted 20 requests and 0.00200000 usage; wallet transactions matched 60.
- Final 60-concurrent evidence covered both providers and all four provider key records:
  - Ekan8 key A: 9 successes, 23 skipped candidates.
  - Ekan8 key B: 28 successes.
  - OpenAI-compatible key A: 17 successes, 10 skipped candidates.
  - OpenAI-compatible key B: 6 successes, 27 skipped candidates.
- Usage flush validation passed: Redis pending and processing usage hashes were empty after flush; `api_tokens.request_count`, `global_models.usage_count`, and wallet billing totals matched successful request count.
- Cold-start recovery validation passed:
  - Unapplied processing batch was flushed exactly once after restart, increasing one token by 2 requests and model usage from 60 to 62.
  - Already-marked processing batch was cleared without double counting.
- Cleanup validation passed after final run: 0 fixture API tokens, 0 fixture request candidates, 0 fixture request records, 0 `real-usage-%` usage flush batches; system settings restored to `load_balance/full/record_request_body=false/record_response_body=false`.
