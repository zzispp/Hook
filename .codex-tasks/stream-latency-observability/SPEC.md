# Stream Latency Observability

## Goal

Expose real stream-stage timing metrics across backend analytics and admin UI without changing stream commit behavior.

## Non-Goals

- Do not add `stream_commit_mode`.
- Do not change stream response commit timing.
- Do not synthesize historical timing values.

## Deliverables

- Backend aggregate storage for response headers, first SSE, first output, and SSE-to-output wait timings.
- API/types for performance monitoring, cost analysis, dashboard overview, user stats, and routing observability.
- Admin UI display for the new timing fields.
- Tests and validation.
