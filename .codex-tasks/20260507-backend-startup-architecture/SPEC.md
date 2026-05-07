# Backend Startup Architecture

## Goal

Review the backend layering and move runtime startup wiring out of `main.rs` so the binary entrypoint remains minimal.

## Scope

- Keep `apps/hook_backend` as the composition root.
- Extract CLI command dispatch from `main.rs`.
- Extract serve-time application wiring into a startup module.
- Keep failures explicit and visible.
- Record architecture boundary findings in the final response.

## Acceptance

- `main.rs` only initializes tracing and delegates execution.
- Commands still support serve, schema bootstrap, schema push, and migration passthrough.
- Startup wiring builds the same routers, middleware, storage, Redis cache, and services.
- Formatting, check, tests, and a short startup validation pass or fail visibly.
