# Billing Access Fix Results

## Fixed Behaviors

- Disabled users are rejected before upstream scheduling with HTTP 403 and `new_api_error`.
- Exhausted user tokens are rejected before upstream scheduling with HTTP 403 and `pre_consume_token_quota_failed`.
- Empty user wallet quota is rejected before upstream scheduling with HTTP 403 and `insufficient_user_quota`.
- Successful LLM billing now settles both API token `used_quota` and user wallet balance.
- Wallet ledger writes `category=consume`, `reason_code=llm_model_usage`, `link_type=llm_request_record`, `link_id=request_id`.
- Wallet ledger `description` stores an immutable JSON snapshot for request, user, token, group, model, provider, endpoint, key, usage, pricing, multiplier, and charged amounts.
- `cache_affinity` cold start now uses load-balance ordering when no cached successful key exists, so equal-priority providers are distributed before affinity is established.

## Real Validation Evidence

- Real script: `.codex-tasks/20260514-billing-access-real-flow/billing_access_real_flow.mjs`.
- Result file: `.codex-tasks/20260514-billing-access-real-flow/raw/results.json`.
- Second real run passed 14/14 scenarios against local DB, local backend, Hook upstream, and Ekan8 upstream.
- Billing request `019e263c-056c-7242-a8f2-81484753ef7f` settled `total_cost=0.07250000`.
- Active token `used_quota` changed from `0.00000000` to `0.07250000`.
- Active wallet recharge balance changed from `10.00000000` to `9.92750000`.
- Active wallet `total_consumed` changed to `0.07250000`.
- Wallet consume amount was `-0.07250000`, linked to `llm_request_record`, with snapshot kind `llm_model_usage`.
- Cold cache-affinity first keys included both `Billing Access Hook A key` and `Billing Access Hook B key`.
- Retry/failover trace showed two broken-provider failures followed by Hook A success.
- Timeout/failover trace showed local slow upstream `upstream_timeout` followed by Hook A success.
- Ekan8 mapped Gemini request succeeded through provider `Billing Access Ekan8`.

## Validation Commands

- `just check`
- `perl -e 'alarm 60; exec @ARGV' cargo test -q -p proxy scheduler`
- `perl -e 'alarm 60; exec @ARGV' cargo test -q -p backend llm_proxy`
- `git diff --check`
- `node --check .codex-tasks/20260514-billing-access-real-flow/billing_access_real_flow.mjs`
- `node --check .codex-tasks/20260514-billing-access-real-flow/lib/billing_access_access_scenarios.mjs`
- `node --check .codex-tasks/20260514-billing-access-real-flow/lib/billing_access_client.mjs`
- `node --check .codex-tasks/20260514-billing-access-real-flow/lib/billing_access_db_control.mjs`
- `HOOK_BILLING_PRIMARY_KEY=... HOOK_BILLING_EKAN8_KEY=... node .codex-tasks/20260514-billing-access-real-flow/billing_access_real_flow.mjs`

## Exposed Non-Code Failure

- One earlier real run exposed a Hook A upstream timeout in the single-provider billing scenario and failed openly with HTTP 502. A full immediate rerun passed the same scenario and all remaining scenarios, so the final evidence uses the successful rerun while preserving the failure as upstream instability evidence.

## Format Check Boundary

- `cargo fmt --all -- --check` currently reports unrelated pre-existing formatting diffs outside this task scope. The command was not used to rewrite the repository because that would touch unrelated files.
