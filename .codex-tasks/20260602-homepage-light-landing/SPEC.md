# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Design and implement a light color treatment for the current React Bits-style landing page.
- Preserve the existing dark visual effect while enabling coherent dark/light support.
- Ground the light palette in the existing home page theme and color usage.

## Non-Goals

- Do not redesign page structure or landing page content.
- Do not add fallback/mock behavior or compatibility paths.
- Do not change backend or admin i18n behavior.

## Constraints

- Follow existing frontend patterns in `apps/hook_frontend`.
- Keep changes scoped to landing page theme and styles unless code structure requires otherwise.
- Validate with frontend lint/build where feasible.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript, JavaScript, React, Next.js
- **Package manager**: pnpm
- **Test framework**: No JS test runner configured; use lint/build validation
- **Build command**: `pnpm build:frontend`
- **Existing test count**: Not measured for frontend

## Risk Assessment

- [x] External dependencies (APIs, services) — no new external dependency needed
- [x] Breaking changes to existing code — scope limited to landing page styling
- [x] Large file generation — not expected
- [x] Long-running tests — frontend validation can be run with explicit timeout

## Deliverables

- Light theme variables and component styles for the landing page.
- Dark theme behavior preserved.
- Validation notes and screenshots/checks where feasible.

## Done-When

- [ ] Landing page has a deliberate light palette based on existing home page colors.
- [ ] Dark and light modes render without obvious contrast or overlap regressions.
- [ ] Frontend lint/build validation is run or any blocker is reported.

## Final Validation Command

```bash
pnpm lint:frontend
```

