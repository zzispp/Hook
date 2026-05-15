# Real Usage Concurrency Flow

## Goal

Write and run a real local integration script that validates the LLM proxy path after usage writes were moved off the customer request path.

The script must use the local PostgreSQL and Redis configured for this project, seed only test-scoped fixtures, create real customer tokens through the running backend API, call real upstream providers, and verify proxy routing, request recording, billing, usage flush, and cold-start recovery.

## Scope

- Insert the requested `menu_sections` rows idempotently.
- Configure real upstream providers from environment variables only.
- Fetch Ekan8 upstream model names at runtime and use the fetched model for mapping.
- Create multiple wallet users and multiple API tokens through admin APIs.
- Exercise high-concurrency traffic across multiple user accounts, tokens, provider keys, and provider formats.
- Verify `request_records`, `request_candidates`, `wallet_transactions`, `api_tokens`, `global_models`, Redis usage pending/processing keys, and `usage_flush_batches`.
- Simulate processing-batch recovery after backend restart for both new and already-applied batches.

## Non-Goals

- Do not persist upstream API keys in files, SQL, or result artifacts.
- Do not drop or reset the local database.
- Do not hide upstream or infrastructure failures behind mock success.

## Required Runtime Env

- Provider 1: `HOOK_REAL_PROVIDER1_KEYS` comma-separated multi-key list, or `HOOK_REAL_PROVIDER1_KEY` for a single key.
- Provider 2: `HOOK_REAL_PROVIDER2_KEYS` comma-separated multi-key list, or `HOOK_REAL_PROVIDER2_KEY` for a single key.

When only the single-key env is present, the script still creates two provider key records for that provider so scheduler and request-record key coverage are tested. The result evidence records only key counts. For distinct upstream credential coverage, use the `*_KEYS` list form.

Optional env:

- `HOOK_REAL_PROVIDER1_BASE` defaults to `https://www.hook.rs`
- `HOOK_REAL_PROVIDER2_BASE` defaults to `https://www.ekan8.com`
- `HOOK_REAL_CONCURRENCY_REQUESTS` defaults to `36`
- `HOOK_REAL_CHAT_MODEL` overrides the global OpenAI-compatible model name.
- `HOOK_REAL_EKAN8_MODEL` overrides the fetched Ekan8 mapped upstream model.

## Run Command

```sh
HOOK_REAL_PROVIDER1_KEYS=key-a,key-b \
HOOK_REAL_PROVIDER2_KEYS=key-c,key-d \
node .codex-tasks/20260515-real-usage-concurrency-flow/real_usage_concurrency_flow.mjs
```

## Evidence

The script writes sanitized evidence to `raw/results.json`. Secrets are never included.
