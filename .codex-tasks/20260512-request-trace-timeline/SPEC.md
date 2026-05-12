# Request Trace Timeline

## Goal

Make Hook request detail tracing follow Aether's provider/key timeline shape while preserving Hook billing group candidate scoping.

## Scope

- Use only candidates produced by the request scheduler after token, billing group, model, provider, endpoint, and key filtering.
- Represent providers as large timeline dots and provider keys/attempts as small dots.
- Distinguish active, unscheduled, queued, not scheduled, success, and failed states.
- Keep request detail drawer usable without adding unrelated routing features.

## Validation

- Rust check/tests for request candidate and record storage.
- Frontend lint/build.
