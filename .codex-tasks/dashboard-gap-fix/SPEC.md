# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Remove the desktop dashboard whitespace caused by the model ranking card being taller than the request trend card.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change dashboard API behavior, data shape, translations, authentication, or chart semantics.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Next.js frontend in `apps/hook_frontend`.
- Keep the fix at the layout composition level; no fallback data or mock success path.
- Preserve mobile single-column rendering.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js
- **Package manager**: pnpm
- **Test framework**: ESLint / Next.js build checks
- **Build command**: `pnpm --filter hook_frontend build`
- **Existing test count**: no frontend unit runner configured

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — local frontend service responded on port 8082.
- [x] Breaking changes to existing code — impact limited to dashboard layout composition.
- [x] Large file generation — no large files generated.
- [x] Long-running tests — lint completed in under 60 seconds.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- `apps/hook_frontend/src/sections/overview/analytics/view/overview-analytics-view.tsx`

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [x] On desktop, request trend and activity grid share the left column so the activity grid starts below the trend card with normal section spacing.
- [x] Model ranking remains in the right column.
- [x] Frontend lint and production build pass.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
pnpm --filter hook_frontend lint && pnpm --filter hook_frontend build
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Open `http://localhost:8082/dashboard/`.
2. Confirm the request trend card is above the activity grid in the left column.
3. Confirm the model ranking card remains in the right column.
