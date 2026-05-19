# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Add a cache affinity TTL input to the provider priority management modal.
- Show the TTL input only when scheduling mode is cache affinity.
- Persist the minutes value to global system settings.
- Make runtime cache affinity TTL read the global system setting instead of provider API keys.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not persist the modal TTL to provider API keys or individual providers.
- Do not edit provider cooldown policy behavior.
- Do not edit frontend locale JSON files.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Next.js frontend with MUI.
- Rust backend and SeaORM storage entities.
- Admin UI copy is backend-seeded from `apps/hook_backend/src/migration/defaults/i18n/`.
- Baseline-only change; no old-data compatibility path is required.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js / Rust
- **Package manager**: pnpm
- **Test framework**: ESLint
- **Build command**: `pnpm build:frontend`
- **Existing test count**: N/A for frontend unit tests

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — existing system settings API and runtime cache snapshot.
- [x] Breaking changes to existing code — scope is system setting, scheduling snapshot, runtime affinity TTL, modal, and i18n seed.
- [x] Large file generation — none.
- [x] Long-running tests — frontend lint only.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- `apps/hook_frontend/src/sections/admin/provider-priority-dialog.tsx`
- `apps/hook_frontend/src/sections/admin/provider-priority-state.ts`
- `apps/hook_frontend/src/actions/system-settings.ts`
- Backend system setting migration/entity/types/storage/validation files.
- Runtime cache affinity snapshot and write path files.
- `apps/hook_backend/src/migration/defaults/i18n/admin.cn.json`
- `apps/hook_backend/src/migration/defaults/i18n/admin.en.json`

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Cache affinity TTL input appears only for `cache_affinity` mode.
- [ ] Empty/default value is 5 minutes.
- [ ] Saving in cache affinity mode updates global `system_settings.cache_affinity_ttl_minutes`.
- [ ] Runtime affinity Redis TTL is taken from the global scheduling snapshot value.
- [ ] Invalid minutes value surfaces a translated validation error.
- [ ] Frontend lint and Rust checks pass or blockers are documented.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
pnpm lint:frontend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Open provider priority management.
2. Select cache affinity.
3. Edit cache affinity minutes and save.
