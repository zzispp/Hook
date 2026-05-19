# Task Specification

## Task Shape
- Shape: single-full

## Goals
- Replace dashboard KPI card decorative glass SVG icons with semantic project-relevant icons.
- Keep the existing KPI data, colors, chart behavior, and layout semantics unchanged.

## Non-Goals
- Do not change dashboard metrics, translations, backend APIs, or visual color palette.
- Do not introduce a new icon library or fallback icon path.

## Constraints
- Project root: /Users/bubu/ZwjProjects/Hook
- Frontend app: apps/hook_frontend
- Use existing Iconify infrastructure and registered icons.
- Keep functions under local code metric limits where touched.

## Deliverables
- Dashboard KPI cards render semantic icons for requests, success rate, tokens, cost, active requests, failures, latency, and models.
- Validation command succeeds or failure is explicitly reported.

## Done-When
- KPI card code no longer references unrelated glass shopping/user/message icons.
- Required Iconify names are registered locally.
- Frontend lint passes for the modified app.

## Final Validation Command
```bash
pnpm --filter hook_frontend lint
```
