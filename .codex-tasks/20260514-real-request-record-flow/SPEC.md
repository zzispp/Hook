# 2026-05-14 Real Request Record Flow

## Goal

Run a real end-to-end Hook proxy validation that covers request-record redesign behavior together with real provider scheduling, failover, conversion, restriction, and concurrency.

## Scope

- Reuse the existing real route and user-access fixtures instead of creating a second fixture system.
- Seed dedicated test providers, endpoints, keys, model bindings, billing group, users, and bearer tokens in the local database.
- Read upstream keys and the generated system token from environment variables only.
- Validate non-streaming, streaming, OpenAI/Claude/Gemini format conversion, fixed order, cache affinity, load balance, provider failover, endpoint fallback, key failover, provider restriction, model restriction, and 100 concurrent requests.
- Validate the current request-record redesign behavior:
  - `request_records` main record fields
  - `request_candidates` `scheduled/skipped` semantics
  - `skip_reason / error_code / error_param`
  - client disconnect -> `cancelled / 499 / client_disconnected`
  - payload compression retention
  - stale pending/streaming sweep

## Constraints

- No mocked upstream success.
- No silent fallback: unexpected HTTP status, missing request rows, missing payloads, or unexpected terminal state fails the run.
- Do not write upstream provider secrets or generated bearer tokens into task files or source files.
- Restore system settings after the run, but keep generated request records in the database so the real evidence remains inspectable.
