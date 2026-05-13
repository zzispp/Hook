# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Adjust the request detail drawer so request headers, request body, and response body use a tab switcher like the aether project.
- Prevent JSON content from being expanded by default so the drawer does not create an excessively long page scroll.

## Non-Goals

- Do not change backend APIs, stored request data, or unrelated admin UI behavior.
- Do not add compatibility fallbacks or mock rendering paths.

## Constraints

- Follow existing Hook frontend component patterns.
- Use the aether implementation only as a behavioral/style reference.
- Keep failures visible and avoid silent fallback behavior.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: TypeScript / Next.js
- **Package manager**: pnpm
- **Test framework**: lint/build validation

## Risk Assessment

- [x] External dependencies (APIs, services) — no new external dependency expected.
- [ ] Breaking changes to existing code — assess affected drawer call sites before coding.
- [ ] Large file generation — not expected.
- [ ] Long-running tests — use frontend lint/build commands; backend tests not relevant.

## Deliverables

- Updated request detail drawer UI.
- Validation output for relevant frontend checks.

## Done-When

- [ ] Request headers/body/response body are displayed through tabs rather than stacked expanded sections.
- [ ] JSON viewers are not expanded by default.
- [ ] Relevant frontend validation passes or any failure is reported with evidence.

## Final Validation Command

```bash
pnpm lint:frontend
```
