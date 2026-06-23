# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape

- **Shape**: `single-full`

## Goals

- Implement stream `first_byte_timeout` fail-fast semantics so a timed-out stream attempt moves directly to the next candidate instead of retrying the same candidate.
- Add single-backup hedged execution for all stream proxy requests, with winner-takes-output semantics based on first effective output.
- Add first-effective-output success metrics into routing observability and make stream routing success scoring use that signal.
- Split stream timing observability into response headers, first SSE event, first effective output, and full completion across backend storage, types, API, and admin UI.

## Non-Goals

- Do not change non-stream request retry semantics.
- Do not change provider-management defaults such as `stream_first_byte_timeout_seconds = 60` in this task.
- Do not redesign routing profile weights or add a new public routing weight field.

## Constraints

- Rust backend + SeaORM storage + Next.js frontend in pnpm monorepo.
- Follow existing request-record compatibility shape: keep `first_byte_time_ms` and `total_latency_ms` available.
- Use additive migrations only.
- Keep failures explicit and observable; no silent fallback behavior.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + TypeScript/React
- **Package manager**: `pnpm`
- **Test framework**: Rust `cargo test`, frontend lint/build validation
- **Build command**: `just build`, `pnpm build:frontend`
- **Existing test count**: existing workspace tests + frontend lint/build checks

## Risk Assessment

- [x] External dependencies (APIs, services) — availability confirmed?
- [x] Breaking changes to existing code — impact assessed?
- [x] Large file generation — disk space sufficient?
- [x] Long-running tests — timeout configured?

## Deliverables

- Backend stream execution changes for fail-fast timeout and hedged streaming.
- Storage/types/migrations for new timing and routing metric fields.
- Admin request record and routing display updates for the new metrics.
- Automated tests covering retry, hedge, timing, and routing behavior.

## Done-When

- [ ] Stream `first_byte_timeout` no longer retries the same candidate.
- [ ] Hedged streaming works for all stream requests with deterministic winner/loser audit behavior.
- [ ] Routing metrics persist and expose first-output success counters and new timing averages.
- [ ] Request record APIs and admin UI surface the new timing fields without breaking existing views.
- [ ] Relevant backend tests and frontend lint/build checks pass.

## Final Validation Command

```bash
cargo test -p hook_backend --lib --tests && cargo test -p storage --lib --tests && pnpm lint:frontend
```

## Demo Flow (optional)

1. Trigger a stream request where the primary candidate stalls before effective output.
2. Observe the backup candidate win and the loser attempt marked as hedge-cancelled.
3. Open request details and verify response-headers / first-event / first-output / completion timings.
