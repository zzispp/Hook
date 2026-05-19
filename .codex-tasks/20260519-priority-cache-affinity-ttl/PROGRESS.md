# Progress Log

> Auto-maintained by Taskmaster. Each entry records what happened, why, and what's next.
> This file serves as both decision audit trail and context-recovery anchor.

---

## Session Start

- **Date**: 2026-05-19 18:15 CST
- **Task name**: `priority-cache-affinity-ttl`
- **Task dir**: `.codex-tasks/20260519-priority-cache-affinity-ttl/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (4 milestones)
- **Environment**: TypeScript / Next.js / ESLint

---

## Context Recovery Block

> If you are resuming this task after compaction, session restart, or context loss,
> read this section FIRST to restore working state.

- **Current milestone**: #4 — Run validation
- **Current status**: DONE
- **Last completed**: #4 — Run validation
- **Current artifact**: Completed global cache affinity TTL setting and priority modal wiring.
- **Key context**: User clarified cache affinity TTL belongs to global scheduling/provider settings, not provider keys. Runtime must read a new global setting value instead of `provider_api_keys.cache_ttl_minutes`.
- **Known issues**: Worktree has unrelated changes. Do not revert them.
- **Next action**: None.

> Update this block EVERY TIME a milestone changes status.

---

<!-- Append entries below as each milestone completes -->

## Milestone 1: Record scope and inspect current APIs

- **Status**: DONE
- **Started**: 18:15
- **Completed**: 18:15
- **What was done**:
  - Inspected priority modal, provider actions, provider key types, backend validation/storage, and i18n seed.
- **Key decisions**:
  - Decision: Persist the modal TTL to provider API keys.
  - Reasoning: Cache affinity writes Redis TTL from `candidate.cache_ttl_minutes`, which is populated from provider key records.
  - Alternatives considered: Adding a system setting, rejected because runtime does not read a global TTL today.
- **Problems encountered**:
  - Problem: None.
  - Resolution: N/A.
  - Retry count: 0
- **Validation**: `rg -n "cache_ttl_minutes" apps/hook_frontend/src crates/types/src crates/provider/src` → exit 0
- **Files changed**:
  - `.codex-tasks/20260519-priority-cache-affinity-ttl/SPEC.md` — recorded scope.
  - `.codex-tasks/20260519-priority-cache-affinity-ttl/TODO.csv` — recorded milestones.
  - `.codex-tasks/20260519-priority-cache-affinity-ttl/PROGRESS.md` — recorded context.
- **Next step**: Milestone 2 — Implement modal TTL input and save path

---

## Scope Correction: Global TTL Target

- **Status**: APPLIED
- **What changed**:
  - User clarified cache affinity time must hang off the global provider scheduling setting, not a specific provider or provider key.
- **Decision**:
  - Add `system_settings.cache_affinity_ttl_minutes` with default `5`.
  - Persist modal input through system settings.
  - Make runtime affinity writes use the global scheduling snapshot TTL.
- **Rejected path**:
  - Do not fetch provider keys or update `provider_api_keys.cache_ttl_minutes` from the priority modal.
- **Next step**: Milestone 2 — Add global cache affinity TTL setting chain

---

## Milestone 2: Add global cache affinity TTL setting chain

- **Status**: DONE
- **Completed**: 18:31
- **What was done**:
  - Added `cache_affinity_ttl_minutes` to baseline `system_settings`, seed data, storage entity, storage patch, settings DTOs, and frontend settings type.
  - Added backend validation that the TTL is greater than 0.
  - Added `cache_affinity_ttl_minutes` to the scheduling snapshot and bumped the Redis scheduling snapshot key from `v2` to `v3` so the new schema is rebuilt explicitly.
  - Changed HTTP proxy and WebSocket affinity writes to use the global snapshot TTL, not provider key `cache_ttl_minutes`.
- **Validation**:
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo check -p backend` → passed with an existing `usage` dead-code warning.
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p setting validate_update_rejects_non_positive_cache_affinity_ttl_minutes` → passed.

## Milestone 3: Wire priority modal and admin i18n

- **Status**: DONE
- **Completed**: 18:31
- **What was done**:
  - Priority modal now receives the global `cache_affinity_ttl_minutes` from system settings.
  - Cache affinity mode shows a minutes input defaulting to 5.
  - Saving cache affinity mode persists `scheduling_mode` and `cache_affinity_ttl_minutes` through `updateSystemSettings`.
  - Removed the wrong provider-key fetch/update path from the modal flow.
  - Added Chinese and English backend-seeded admin i18n keys.
- **Validation**:
  - `pnpm lint:frontend` → passed.
  - `python -m json.tool apps/hook_backend/src/migration/defaults/i18n/admin.cn.json >/dev/null && python -m json.tool apps/hook_backend/src/migration/defaults/i18n/admin.en.json >/dev/null` → passed.

## Milestone 4: Run validation

- **Status**: DONE
- **Completed**: 18:31
- **Validation summary**:
  - Frontend lint passed.
  - Backend cargo check passed.
  - New setting validation test passed.
  - i18n JSON files parse successfully.

<!-- Final summary goes here when all milestones are DONE -->

## Final Summary

- **Total milestones**: 4
- **Completed**: 4
- **Failed + recovered**: 0
- **External unblock events**: 1
- **Total retries**: 0
- **Files created**: 1
- **Files modified**: 30+
- **Key learnings**:
  - Cache affinity TTL is now a global scheduling setting.
  - Provider key `cache_ttl_minutes` remains available for key metadata/billing contexts but no longer drives affinity Redis TTL.
