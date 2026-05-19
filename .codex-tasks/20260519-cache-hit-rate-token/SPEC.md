# Cache Hit Rate Token Semantics

## Goal

Change Hook performance monitoring cache hit rate to match Aether dashboard semantics.

## Scope

- Backend performance monitoring aggregation only.
- Preserve the existing API field name and frontend display.
- Cache hit rate remains a ratio value for the frontend formatter.

## Acceptance

- `cache_hit_rate` is calculated from cache read tokens over input context tokens.
- Input context tokens are non-cache prompt tokens plus cache read tokens.
- Existing performance monitoring tests pass, with a targeted test covering the new ratio.
