# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Fix the provider cooldown policy modal so cooldown duration is configured either once for all rules or separately per rule.
- Save a fixed one-minute failure window in the existing backend payload shape.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change backend cooldown policy types or cooldown execution semantics.
- Do not touch the unrelated stream idle timeout work.
- Do not add frontend locale JSON files.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Next.js frontend with MUI components.
- Admin UI translations are seeded from backend i18n JSON defaults.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js
- **Package manager**: pnpm
- **Test framework**: ESLint / Next build checks
- **Build command**: `pnpm build:frontend`
- **Existing test count**: N/A for frontend unit tests

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — no external calls required.
- [x] Breaking changes to existing code — limited to modal payload assembly and labels.
- [x] Large file generation — no generated assets.
- [x] Long-running tests — frontend lint/build only.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- `apps/hook_frontend/src/sections/admin/provider-cooldown-policy-dialog.tsx`
- `apps/hook_backend/src/migration/defaults/i18n/admin.cn.json`
- `apps/hook_backend/src/migration/defaults/i18n/admin.en.json`

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Modal has an explicit cooldown duration mode with fixed-all and per-rule choices.
- [ ] Failure window is not editable and saves as 60 seconds when rules exist.
- [ ] Fixed mode saves all rules with the same `cooldown_seconds`.
- [ ] Per-rule mode requires and saves each rule's own `cooldown_seconds`.
- [ ] Labels distinguish failure window from cooldown duration.
- [ ] Frontend validation passes.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
pnpm lint:frontend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Open provider management.
2. Open cooldown policy.
3. Toggle fixed/per-rule cooldown duration mode and save a policy.
