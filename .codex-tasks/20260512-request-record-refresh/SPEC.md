# Request Record Refresh

## Goal

Make request record auto refresh behave like Aether:

- Auto refresh must not make the manual refresh buttons flicker.
- Active request polling should merge incremental request data without regressing visible metrics.
- Streaming requests should expose first-byte time while still in progress, then expose total latency after completion.

## Scope

- Request records frontend refresh state and active polling merge.
- LLM proxy streaming attempt recording.
- Focus on real request-record behavior; no mock success paths.

