# Audit Summary

## Verified Source Paths

- Proxy authentication: `apps/hook_backend/src/llm_proxy/auth.rs`
- Proxy request preparation: `apps/hook_backend/src/llm_proxy/proxy/request.rs`
- Candidate selection and scheduling: `apps/hook_backend/src/llm_proxy/candidate/selection.rs`
- Scheduler ordering: `crates/proxy/src/scheduler/builder.rs`
- Retry and timeout execution: `apps/hook_backend/src/llm_proxy/proxy/executor.rs`
- Request billing and usage persistence: `apps/hook_backend/src/llm_proxy/audit.rs`
- Billing calculation: `crates/provider/src/application/billing.rs`
- API token usage update: `crates/storage/src/api_token/repository.rs`
- User and wallet records: `crates/storage/src/user/repository.rs`, `crates/storage/src/wallet/repository.rs`
- New API insufficient-balance reference: `/Users/bubu/ZwjProjects/new-api/service/billing_session.go`, `/Users/bubu/ZwjProjects/new-api/service/quota.go`, `/Users/bubu/ZwjProjects/new-api/middleware/auth.go`

## Access And Balance Findings Before Real Script

- API token disabled state is enforced in proxy auth: `validate_token` rejects `!token.is_active` with HTTP 401.
- API token expiry is enforced in proxy auth with HTTP 401.
- User disabled state is not enforced in the LLM proxy path found in this audit. The scheduling snapshot loads user access fields but not `users.is_active`, and `select_candidates` only uses user model/provider/rate-limit fields.
- API token quota limit is present on `api_tokens.quota_limit`, but the cached token decoder intentionally reconstructs `used_quota` as `0`, and no request-preparation check compares `used_quota` with `quota_limit`.
- User wallet balance and wallet `limit_mode/status` are not referenced by the LLM proxy request path found in this audit.
- Successful requests call `state.tokens.record_usage`, which increments `api_tokens.used_quota`, `request_count`, and `last_used_at`.
- No wallet consume ledger path was found on successful LLM proxy billing. Wallet service currently covers balance reads, admin recharge, and admin adjust.

## Billing Multiplier Findings Before Real Script

- Candidate construction sets `billing_multiplier` from the token's billing group.
- `calculate_request_billing` computes `base_cost = token_cost + price_per_request` and `total_cost = base_cost * billing_multiplier`, rounded to 8 decimals.
- Request records and request candidates persist `token_cost`, `base_cost`, `total_cost`, and `billing_multiplier`.
- API-token usage persists the same `total_cost` into `api_tokens.used_quota`.

## Retry, Timeout, And Scheduling Findings Before Real Script

- Provider request timeout is applied through `reqwest::RequestBuilder::timeout`. Streaming uses `stream_first_byte_timeout_seconds`; non-streaming uses `request_timeout_seconds`.
- Retry loops run `0..=candidate.max_retries`. `max_retries` is the configured provider/endpoint value raised to at least the route option floor for endpoint/key fallback.
- Retryable upstream HTTP statuses are 5xx, 401, 403, 408, and 429.
- `CacheAffinity` promotes a cached key if present; without a cached key it leaves the stable priority order intact.
- `LoadBalance` applies stable per-request hashing inside equal priority buckets.

## New API Comparison Points

- New API token auth rejects disabled users with HTTP 403 and OpenAI-shaped body `{ "error": { "type": "new_api_error", ... } }`.
- New API insufficient user quota returns HTTP 403 with error code `insufficient_user_quota`.
- New API insufficient token quota returns HTTP 403 with error code `pre_consume_token_quota_failed`.
