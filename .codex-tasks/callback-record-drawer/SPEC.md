# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Make admin payment callback records behave like request records: clicking a callback row opens a right-side drawer.
- Move callback raw params and error message out of the table into the drawer.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change backend API shape or storage.
- Do not add frontend locale JSON files.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Follow existing admin Material UI patterns.
- Use backend-controlled `admin` namespace translations.
- Keep changed functions under repository complexity limits.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js
- **Package manager**: pnpm
- **Test framework**: frontend lint/build checks
- **Build command**: `pnpm build:frontend`
- **Existing test count**: no JS test runner configured

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — no new dependency needed
- [x] Breaking changes to existing code — frontend-only list/detail interaction
- [x] Large file generation — none
- [x] Long-running tests — lint only unless build is needed

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- `apps/hook_frontend/src/sections/recharge/recharge-callback-detail-drawer.tsx`
- Updated callback table row interaction.
- Updated recharge management page selection state.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Clicking a callback row opens a drawer.
- [ ] Drawer shows raw params and error message.
- [ ] Frontend lint succeeds or any failure is reported with command output.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
pnpm lint:frontend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Open admin recharge management.
2. Select the callback records tab.
3. Click a callback row and inspect the right drawer.
