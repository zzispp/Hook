# Fixture And Validation Matrix

## Dedicated Fixture Namespace

- IDs use `00000000-0000-7000-9300-*`.
- Billing groups:
  - `billing_access_real` with multiplier `2.5`
  - `billing_access_real_low` with multiplier `1`
- Global model:
  - Local test model name resolves from `HOOK_OPENAI_MODEL` when present, otherwise an existing active `gpt-*` model.
- Providers:
  - `Billing Access Hook A`, real OpenAI-compatible upstream, priority `10`, timeout `45s`, max retries `0`.
  - `Billing Access Hook B`, real OpenAI-compatible upstream, priority `10`, timeout `45s`, max retries `0`.
  - `Billing Access Broken`, non-routable local upstream, priority `0`, max retries `0`.
  - `Billing Access Slow`, black-hole local upstream, priority `0`, request timeout `0.2s`, max retries `0`.
- API tokens:
  - `active`: normal token in multiplier group.
  - `disabled_token`: `is_active=false`.
  - `disabled_user`: user `is_active=false`.
  - `token_quota_exhausted`: `quota_limit=0`, `used_quota=0`.
  - `user_wallet_exhausted`: finite wallet with zero balance.
  - `routing`: normal token for route strategy checks.

## Real Request Assertions

- Disabled token must be rejected before creating request records. Expected current implementation: HTTP 401.
- Disabled user should be rejected before upstream. Audit found no proxy-side user status check, so the script marks a 200/upstream attempt as an explicit failed assertion.
- Token quota exhausted should match New API behavior: HTTP 403 with code `pre_consume_token_quota_failed`. Audit found no pre-consume check, so a 200/upstream attempt is an explicit failed assertion.
- User wallet exhausted should match New API behavior: HTTP 403 with code `insufficient_user_quota`. Audit found no pre-consume wallet check, so a 200/upstream attempt is an explicit failed assertion.
- Successful billing request must persist `billing_multiplier=2.5`, and `total_cost = base_cost * 2.5`.
- API token `used_quota` must increase by the successful request's `total_cost`.
- Wallet balance/transactions must be inspected after successful LLM billing. The current expected audit finding is no wallet deduction and no `consume` ledger entry.

## Routing Assertions

- Retry/failover:
  - Enable broken provider with higher priority and real provider behind it.
  - A request should record the broken candidate as failed and then the real provider as success.
- Timeout:
  - Enable slow provider with `request_timeout_seconds=0.2` before real provider.
  - A request should record `upstream_timeout` or send/read timeout evidence before real success.
- Load balance:
  - With two real providers at equal priority under `load_balance`, several requests should distribute first successful provider/key across both candidates.
- Cache affinity:
  - With `cache_affinity`, the first successful request should remember the successful key.
  - Follow-up requests should select the cached provider/key first.
  - Without an existing cache entry, current source code keeps stable priority order; it does not randomize equal-priority candidates.

## Upstream Model Mapping

- Ekan8 model discovery is a separate probe through `/v1/models` using the Ekan8 key.
- If the probe returns an OpenAI-compatible Claude/Gemini alias, the script records the discovered names and can map the local global model to a discovered alias for a real request.
- Missing or drifting upstream models fail visibly; no mock success is inserted.
