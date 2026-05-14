# Implementation Notes

## Verified Root Causes

- Disabled users are not enforced because `CachedUserAccess` does not carry `users.is_active`, and `ensure_user_allows_model` only checks allow-lists.
- Token quota is not enforced before scheduling; the auth cache also serializes tokens without `used_quota`, so quota decisions cannot use cached tokens correctly.
- User wallet quota is not enforced before scheduling, and successful LLM billing only updates request records and API token usage.
- Wallet ledger needs an immutable settlement snapshot because request IDs alone lose audit context if models, providers, groups, keys, or tokens are changed/deleted.
- `cache_affinity` cold start currently keeps stable priority order. Without an affinity key, equal-priority providers are not distributed.

## Fix Strategy

- Add explicit preflight access checks before candidate scheduling: user active, token quota, wallet active/non-empty when user quota mode is `wallet`.
- Use New API-aligned 403 error codes for disabled user, token quota, and wallet quota.
- Add successful settlement wallet consumption with `consume / llm_model_usage / llm_request_record` and JSON description snapshot.
- Preserve and invalidate token usage in auth cache after successful billing usage is recorded.
- Make scheduler cache-affinity fall back to load-balance ordering when no affinity key exists.
