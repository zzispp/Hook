# Candidate Routing Compaction Epic

## Goal

Remove candidate explosion at the scheduling model level, not only in trace display. The proxy should schedule provider-level routes, resolve endpoint and key choices as execution strategy, and record the actual attempt path with explicit terminal states.

## Current Baseline

- Retry pre-expansion has already been removed in `.codex-tasks/20260512-trace-candidate-compaction`.
- Successful requests now mark remaining `available` rows as `unused`.
- Frontend trace display currently compresses unexecuted rows, but the backend still builds `endpoint x key` candidate parts.

## Target Invariants

- Multi-key providers produce one route per provider or endpoint policy, not one candidate per key.
- Conversion endpoints are fallback paths, not peers of exact-format endpoints in the visible primary route.
- Every real attempt records the actual provider, endpoint, key, retry index, and conversion path.
- Completed requests leave no dangling `available` records after success or terminal failure.
- Trace UI represents `provider -> endpoint -> key -> retry` with summary counts for unused alternatives.
- Any topK or budget behavior is explicit, configured, and auditable. No silent hard-coded caps.

## Non-Goals

- Do not add mock upstream success paths.
- Do not silently degrade to a smaller candidate set without recording why.
- Do not auto-migrate historical trace facts unless a separate explicit maintenance command is added.
