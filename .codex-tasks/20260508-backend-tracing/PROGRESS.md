# Progress Log

## Session Start

- **Date**: 2026-05-08
- **Task name**: `20260508-backend-tracing`
- **Scope**: extract a shared tracing crate and wire backend startup logging through it.

## Milestone 1: Inspection

- **Status**: DONE
- **Reference pattern**:
  - The core project keeps a dedicated `gem_tracing` crate with `init_global_subscriber()`, field-formatting helpers, and startup logging macros.
  - The current Hook backend only initializes `tracing_subscriber::fmt::init()` in `main.rs` and emits a single `tracing::info!` in `startup.rs`.

## Milestone 2: Implementation

- **Status**: DONE
- **Completed**:
  - Add a `hook_tracing` workspace crate.
  - Move subscriber initialization into that crate.
  - Switch backend startup logging to the shared crate so startup output is visible.

## Milestone 3: Validation

- **Status**: DONE
- `cargo check`: passed.
- `cargo fmt --all -- --check`: passed after applying `cargo fmt --all`.
- `cargo clippy -p backend --all-targets -- -D warnings`: passed.
- `just test`: passed.
- `cargo run -p backend`: short runtime verification passed with visible `backend starting` and `backend listening` logs under `RUST_LOG=info`.
- Runtime note: the current shell had `RUST_LOG=warn`, which intentionally hides `info` startup logs.

## Milestone 4: Configurable Tracing

- **Status**: DONE
- **New requirement**:
  - Backend logging must use the internal `hook_tracing` crate.
  - Tracing settings, including log level, must be configurable from project config.
  - Invalid tracing config should fail explicitly instead of silently falling back.
- **Completed**:
  - Added `tracing.log_level` to typed configuration and `config/config.yaml`.
  - Backend initializes `hook_tracing` after loading config.
  - Added backend logging constraints to `apps/hook_backend/AGENTS.md`.

## Milestone 5: HTTP Request Tracing

- **Status**: DONE
- **Requirement**:
  - Add Axum/Tower request tracing so HTTP requests emit tracing events.
  - Use `tower-http` tracing middleware rather than manual per-handler logs.
- **Completed**:
  - Enabled the `tower-http` `trace` feature.
  - Added `TraceLayer::new_for_http()` to the backend router.
  - Verified `GET /health` emits request start and finish debug logs.

## Milestone 6: Model Directory Access Split

- **Status**: DONE
- **Requirement**:
  - Remove the user-facing model directory from the admin default seed.
  - Keep admin model management APIs and menu items intact.
- **Completed**:
  - Admin default API bindings now exclude `models_public_catalog_read`.
  - Admin default menu bindings now exclude `dashboard_models`.
  - Added a backend test to lock the split in place.
