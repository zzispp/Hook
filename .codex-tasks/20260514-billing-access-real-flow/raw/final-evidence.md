# Billing Access Real Flow Evidence

## Validation

- Static checks passed:
  - `node --check .codex-tasks/20260514-billing-access-real-flow/billing_access_real_flow.mjs`
  - `node --check .codex-tasks/20260514-billing-access-real-flow/lib/billing_access_access_scenarios.mjs`
  - `node --check .codex-tasks/20260514-billing-access-real-flow/lib/billing_access_client.mjs`
  - `node --check .codex-tasks/20260514-billing-access-real-flow/lib/billing_access_db_control.mjs`
- Real DB/upstream script ran and wrote `.codex-tasks/20260514-billing-access-real-flow/raw/results.json`.
- After the backend fixes, the real script passed 14/14 scenarios against local DB, local backend, Hook upstream, and Ekan8 upstream.

## Upstream Models

- Hook upstream `/v1/models` returned 11 models.
- Selected Hook provider model: `gpt-5.4-mini`.
- Ekan8 upstream `/v1/models` returned 36 models.
- Selected Ekan8 mapped provider model: `[满血A]gemini-3.1-pro-preview`.

## Access Limits

- Disabled token is enforced: HTTP 401, no request candidate was created.
- Disabled user is enforced before upstream scheduling: HTTP 403 with `new_api_error`, no request candidate was created.
- Token quota exhaustion is enforced before upstream scheduling: HTTP 403 with `pre_consume_token_quota_failed`, no request candidate was created.
- Wallet quota exhaustion is enforced before upstream scheduling: HTTP 403 with `insufficient_user_quota`, no request candidate was created.

## Billing

- Billing group multiplier is applied.
- Evidence request `019e263c-056c-7242-a8f2-81484753ef7f`:
  - `base_cost = 0.02900000`
  - `billing_multiplier = 2.50000000`
  - `total_cost = 0.07250000`
- The active API token `used_quota` increased from `0.00000000` to `0.07250000`, matching `total_cost`.
- Wallet charging is connected to LLM billing:
  - wallet recharge balance changed from `10.00000000` to `9.92750000`
  - wallet `total_consumed` changed to `0.07250000`
  - wallet transaction amount was `-0.07250000`
  - wallet transaction uses `consume / llm_model_usage / llm_request_record`
  - wallet transaction description contains the immutable LLM settlement snapshot

## Provider Retry, Timeout, And Routing

- Provider retry is effective: broken provider at higher priority was attempted twice, then Hook A succeeded.
- Provider timeout is effective: slow local upstream produced `upstream_timeout`, then Hook A succeeded.
- `load_balance` with equal priority distributed requests across Hook A and Hook B in 12 real requests.
- Warm `cache_affinity` is effective: after first success, the cached key was reused.
- Cold `cache_affinity` equal-priority random selection is effective after the fix: six cold starts selected both `Billing Access Hook A key` and `Billing Access Hook B key`.
- Ekan8 mapped request succeeded through the Gemini provider with the mapped model `[满血A]gemini-3.1-pro-preview`.

## New API Reference

- New API disabled-user token auth reference: HTTP 403.
- New API insufficient user quota reference: HTTP 403 with `insufficient_user_quota`.
- New API insufficient token quota reference: HTTP 403 with `pre_consume_token_quota_failed`.
