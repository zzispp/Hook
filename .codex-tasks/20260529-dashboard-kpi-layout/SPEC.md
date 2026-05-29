# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Remove the admin-only Active users large KPI card from the top dashboard KPI row.
- Add Active users as the same style period summary card immediately to the right of the period transfer/failover count card.
- Add Cache hit rate as a standalone admin-only top KPI card.
- Keep Today's cost detail focused on upstream cost and do not repeat Cache hit rate there.
- Remove the upstream cost detail from Today's cost top KPI.
- Replace the admin period transfer/failover count card with an upstream cost card.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not change backend metric semantics or add fallback/mock dashboard data.
- Do not introduce frontend locale JSON files; dashboard copy remains backend i18n controlled.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Follow existing Next.js/MUI dashboard patterns.
- Keep the change scoped to dashboard KPI card configuration unless source inspection proves layout changes are required.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: `TypeScript / Next.js frontend`
- **Package manager**: `pnpm`
- **Test framework**: `ESLint and Next.js build validation`
- **Build command**: `pnpm build:frontend`
- **Existing test count**: `No frontend test runner configured`

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — not required for static card order change.
- [x] Breaking changes to existing code — limited to display order/config.
- [x] Large file generation — not applicable.
- [x] Long-running tests — frontend lint/build only if needed.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Updated dashboard KPI card configuration.
- Updated dashboard period summary card configuration.
- Validation output recorded in `PROGRESS.md`.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [x] Admin top KPI row no longer has the large Active users card.
- [x] Admin period summary row shows Active users immediately after transfer/failover count.
- [x] Admin top KPI row has a standalone Cache hit rate card.
- [x] Today's cost no longer repeats Cache hit rate in its detail.
- [x] Today's cost no longer shows upstream cost in its detail.
- [x] Admin period summary row uses upstream cost instead of transfer/failover count.
- [x] Frontend static validation passes for touched files.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
pnpm --filter hook_frontend exec eslint src/sections/overview/analytics/view/dashboard-kpi-config.ts src/sections/overview/analytics/view/dashboard-period-summary.tsx
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1.
