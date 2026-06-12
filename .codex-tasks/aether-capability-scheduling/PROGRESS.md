# Progress Log

## Session Start

- **Date**: 2026-06-12
- **Task name**: aether-capability-scheduling
- **Task dir**: `.codex-tasks/aether-capability-scheduling/`
- **Environment**: Rust backend / Next.js admin UI / cargo + pnpm validation

## Context Recovery Block

- **Current milestone**: #4 — Run verification
- **Current status**: DONE
- **Last completed**: #4 — Run verification
- **Current artifact**: `TODO.csv`
- **Key context**: Aether uses provider-key `key_capabilities` for required capability checks. Hook currently has global model `supported_capabilities` but no provider-key capability field.
- **Known issues**: Worktree already contains unrelated prior response/image-routing modifications; do not revert them.
- **Next action**: Report completion.

## Milestone 1: Wire capability data through storage/cache/request

- **Status**: DONE
- **Validation**: `timeout 300 cargo check -p hook_backend` → exit 0
- **Files changed**:
  - backend storage/API/cache/request fields for provider key capabilities and request required capability.

## Milestone 2: Implement hard candidate filtering

- **Status**: DONE
- **Validation**: `timeout 60 cargo test -p hook_backend matching_candidate_parts_requires` → exit 0
- **Files changed**:
  - `apps/hook_backend/src/llm_proxy/candidate/selection/matching.rs` — filters by global model and provider key capabilities.

## Milestone 3: Expose provider key capability configuration

- **Status**: DONE
- **Validation**: `pnpm lint:frontend` → exit 0
- **Files changed**:
  - provider key admin form/types/i18n now expose `image_generation` capability.

## Milestone 4: Run verification

- **Status**: DONE
- **Validation**:
  - `cargo fmt --check` → exit 0
  - `timeout 60 cargo test -p hook_backend matching_candidate_parts_requires` → exit 0
  - `timeout 60 cargo test -p hook_backend matching_candidate_parts_routes` → exit 0
  - `timeout 300 cargo check -p hook_backend` → exit 0
  - `pnpm lint:frontend` → exit 0
- **Final cleanup**:
  - Added the shared `IMAGE_GENERATION_CAPABILITY` constant and routed image capability checks through it.
  - Re-ran the same validation set after cleanup; all commands passed.

## Final Summary

- **Total milestones**: 4
- **Completed**: 4
- **Failed + recovered**: 2
- **External unblock events**: 0
- **Total retries**: 2
- **Files created**: 1 migration file plus task records
- **Key learnings**:
  - Hook had global model capabilities but no provider key capabilities; Aether's hard filter maps cleanly after adding explicit key capabilities.
