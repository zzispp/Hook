# 2026-05-12 Real Proxy Cache Flow

## Goal

Run real end-to-end proxy tests against the local Hook backend and real upstream providers, covering non-streaming, streaming, format conversion, scheduling strategies, failover, high concurrency, and the Redis-backed proxy cache module.

## Scope

- Seed deterministic real-test providers, endpoints, keys, model bindings, group bindings, and a system token into the local database.
- Use environment variables for all secrets. Do not write provider keys or system tokens into tracked files.
- Exercise:
  - fixed order
  - failover retry/transfer
  - cache affinity
  - load balance
  - OpenAI exact
  - OpenAI-to-Claude conversion
  - OpenAI-to-Gemini conversion
  - Claude-to-OpenAI conversion
  - Gemini exact
  - non-streaming and streaming calls
  - concurrent non-streaming and streaming calls
  - scheduling snapshot Redis cache
  - auth token Redis cache and auth version bump
  - affinity Redis cache
  - API-driven cache hook for system settings

## Constraints

- No mocked provider success path.
- No silent fallback. Every failed upstream call must fail the test unless it is the expected broken-provider failover attempt.
- Direct database changes that affect scheduling must explicitly clear the scheduling snapshot cache because they bypass API CUD hooks.
- Avoid holding database transactions while rebuilding or inspecting Redis cache state.

