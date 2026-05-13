# 2026-05-13 User Access Real Flow

## Goal

Run real end-to-end Hook proxy traffic after adding user-level allowed provider and allowed model restrictions.

## Scope

- Reuse the existing real route scheduler fixture style.
- Seed dedicated test users, user tokens, providers, endpoints, keys, model bindings, and billing group in the local database.
- Read upstream provider keys and generated bearer tokens from environment variables only.
- Exercise fixed order, cache affinity, load balance, provider failover, key failover, endpoint fallback, format conversion, non-streaming, streaming, and 100 concurrent requests.
- Assert user-level allowed model and allowed provider restrictions are enforced for user tokens and do not silently fall back to disallowed candidates.
- Assert default empty user limits mean unrestricted access.

## Constraints

- No mocked upstream success.
- No silent fallback: unexpected HTTP status, missing request candidate rows, or unexpected trace state fails the run.
- Do not write raw upstream provider keys or bearer token values into task files or source files.
- Do not reset the local database. Test setup may apply non-destructive `ADD COLUMN IF NOT EXISTS` for newly added user access columns when the local dev schema predates this code change.
