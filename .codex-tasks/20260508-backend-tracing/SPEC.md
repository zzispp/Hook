# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Extract a shared `hook_tracing` crate from the reference tracing pattern.
- Use the shared crate in the backend startup path.
- Make backend startup emit visible logs without requiring manual wiring at each call site.

## Non-Goals

- Do not add Sentry or other error-reporting integrations.
- Do not change backend business logic, API behavior, or persistence behavior.

## Done-When

- `hook_tracing` exists as a workspace crate.
- Backend initialization uses `hook_tracing::init_global_subscriber()`.
- Backend startup logs are visible on `cargo run -p backend`.
- `cargo check`, `cargo test`, and the backend runtime verification complete successfully or any failure is recorded explicitly.

