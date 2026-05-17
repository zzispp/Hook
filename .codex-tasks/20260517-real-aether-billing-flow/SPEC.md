# Real Aether Billing Flow

## Goal

Run a real end-to-end Hook proxy validation against local Postgres/Redis and real upstream providers. The test must verify the new Aether-style billing chain, billing group multiplier, request/candidate billing snapshots, wallet settlement, token usage, and model usage.

## Boundaries

- Use local database container `hook-postgres` and Redis `127.0.0.1:6380`.
- Use real upstream providers from environment variables only.
- Do not write provider keys or generated bearer tokens to source files or task artifacts.
- Do not mock provider responses or fabricate success.
- Fail explicitly when schema, upstream, backend, billing, wallet, or audit expectations do not match.

## Required Environment

- `HOOK_86GAMESTORE_KEY`
- `EKAN8_KEY`

Optional overrides:

- `HOOK_86GAMESTORE_BASE_URL`
- `EKAN8_BASE_URL`
- `HOOK_BACKEND_URL`
- `HOOK_PG_CONTAINER`

## Validation Surface

- Provider model discovery through `/v1/models`.
- Real OpenAI chat completion through Hook proxy.
- Ekan8 mapped model runtime through Hook proxy.
- `billing_rules` and `dimension_collectors` driven billing.
- `BillingSnapshot` persisted on `request_records` and `request_candidates`.
- `billing_status = settled` for successful complete billing.
- Group multiplier applied after base cost.
- Wallet finite balance reduced by total cost.
- Token `used_quota` and `request_count` updated.
- Global model `usage_count` updated.
