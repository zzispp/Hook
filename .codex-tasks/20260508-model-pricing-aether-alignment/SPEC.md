# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Align Hook admin global model pricing with Aether's tiered pricing editor.
- Ensure model.dev selection produces cache creation, cache read, and 1h cache prices using Aether's pricing rules.
- Fix successful model deletion being reported as `Response data not found`.

## Non-Goals

- Do not change backend API contracts unless the existing contract cannot store tiered pricing.
- Do not add mocked model.dev data or silent success paths.
- Do not redesign unrelated model management UI.

## Done-When

- Admin create/edit supports adding/removing price tiers with an unlimited final tier.
- Submitted tiers include cache creation/read and 1h cache values.
- DELETE success updates the model table without a false error toast.
- Relevant frontend validation passes, or failures are documented with root cause.
