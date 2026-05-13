# 2026-05-13 Real Route Scheduler Flow

## Goal

Run real end-to-end proxy traffic through the local Hook backend after the provider-route candidate refactor.

## Scope

- Seed dedicated real-test providers, endpoints, keys, model bindings, billing group, and token into the local database.
- Read all tokens and provider keys from environment variables only.
- Exercise fixed order, provider failover, route key failover, route endpoint fallback, cache affinity, load balance, format conversion, streaming, non-streaming, and high concurrency.
- Assert request_candidates rows reflect real attempts only, not endpoint x key x retry prebuilt dots.

## Constraints

- No mocked upstream success.
- No silent fallback: every unexpected upstream or assertion failure fails the run.
- Direct DB scheduling changes must clear the Redis scheduling snapshot.
- Do not write raw tokens or provider keys into task files or source files.
