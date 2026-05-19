# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Fix dashboard overview PostgreSQL query results so latency averages decode into Rust `Option<f64>` without SQL type mismatch.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change public dashboard response types.
- Do not add fallback decoding or mock success behavior.
- Do not alter unrelated dashboard metrics.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Rust storage crate with SeaORM/PostgreSQL raw SQL.
- `request_records.total_latency_ms` and `first_byte_time_ms` are integer columns mapped as `Option<i64>`.
- PostgreSQL `AVG(bigint)` returns `NUMERIC`; SQL must return `double precision` for Rust `Option<f64>` rows.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace, SeaORM/PostgreSQL SQL queries
- **Package manager**: pnpm for workspace scripts
- **Test framework**: cargo test, pnpm check:backend
- **Build command**: `pnpm check:backend`
- **Existing test count**: storage crate has focused integration tests under `crates/storage/tests/`

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — no real database required for focused SQL generation test.
- [x] Breaking changes to existing code — limited to dashboard overview latency aggregate SQL.
- [x] Large file generation — not applicable.
- [x] Long-running tests — 60 second cargo test wrapper configured.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- `crates/storage/src/dashboard/overview.rs` AVG latency expressions return SQL `double precision`.
- Focused storage test coverage for dashboard overview SQL and f64 row mapping.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Dashboard overview no longer emits NUMERIC latency averages for Rust `Option<f64>` fields.
- [ ] Targeted tests pass with the repository's 60 second backend timeout rule.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
perl -e 'alarm 60; exec @ARGV' cargo test -p storage dashboard && pnpm check:backend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Request `/api/dashboard/overview` against a Postgres database with request latency records.
2. Response includes numeric `avg_latency_ms` / `avg_ttfb_ms` instead of infrastructure decode error.
