# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Stop streaming request records from staying `streaming/pending` when the upstream produces no chunks after the first byte.
- Preserve billing for completed streams that lack provider usage when Hook has enough streamed output to estimate tokens.
- Keep the behavior explicit and provider-configurable.

## Non-Goals

- Do not manually modify production request records.
- Do not add mock success paths or hidden fallbacks.
- Do not change frontend display behavior.

## Constraints

- Follow existing Rust workspace patterns.
- Keep stream timeout semantics separate from non-stream request timeout.
- Expose failures as explicit timeout/failure records.
- Run backend checks with the repository timeout wrapper or a 60-second shell timeout.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024 workspace
- **Package manager**: pnpm for frontend, Cargo/Just for backend
- **Test framework**: Rust unit tests
- **Build command**: `just check`

## Risk Assessment

- [x] External dependencies: production DB inspected read-only.
- [x] Breaking changes: provider schema/type changes affect API, storage, cache, selection, and tests.
- [x] Long-running tests: use `timeout 60`.

## Deliverables

- Provider-level `stream_idle_timeout_seconds` configuration flowing into `ProxyCandidate`.
- Stream relay idle timeout after first byte, recording failed terminal state on timeout.
- Usage estimation behavior verified for EOF-with-output cases.
- Focused backend tests.

## Done-When

- [ ] A stream with no upstream chunk after the configured idle window records a terminal failure.
- [ ] Existing stream usage estimation still settles EOF streams that have output deltas.
- [ ] `just check` or focused Rust tests pass within timeout.

## Final Validation Command

```bash
timeout 60 just test
```
