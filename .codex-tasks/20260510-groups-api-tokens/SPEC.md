# Groups And API Tokens

## Goal

Implement billing groups and user API tokens in Hook. Billing groups control billing multiplier and will later connect channels. API tokens are user-owned credentials bound to a billing group and model access policy.

## Scope

- Add backend storage, migrations, domain services, and HTTP APIs for billing groups.
- Add backend storage, migrations, domain services, token generation/hash, and HTTP APIs for API tokens.
- Add frontend management screens for admin billing groups and user API tokens.
- Seed default billing group and RBAC menu/API entries.

## Business Semantics

- Users do not have groups.
- API tokens bind to one billing group.
- Billing formula is model price multiplied by group billing multiplier.
- Channel and API format access controls are reserved because those modules do not exist yet.
- Token defaults:
  - quota limit: unlimited
  - allowed models: all models
  - rate limit: follow system
- System rate settings do not exist yet, so token rate limits are persisted/displayed only.

## Validation

- Rust formatting and cargo checks must pass.
- Frontend lint/build checks should run when feasible.
- No mock success path or silent fallback is allowed.
