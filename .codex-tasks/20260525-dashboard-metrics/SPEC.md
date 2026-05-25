# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Update the dashboard API and UI so admin and user KPI cards show requests, success rate, Tokens, cost, failed, average response, cache hit rate, and TTFB.
- Remove Active, Models used, and API format dashboard cards.
- Add activity-grid hover content with date, requests, Tokens, billed cost, and base cost.
- Keep admin Provider distribution visible and add request count, Tokens, average response, and cost details.

## Non-Goals

- Do not change request recording, billing formulas, request records pages, or performance monitoring behavior.
- Do not add frontend admin locale JSON files.

## Constraints

- Follow repository AGENTS.md: Chinese user-facing final reply, Debug-First, no silent fallbacks.
- Backend source uses Rust 2024 and Postgres SQL through SeaORM raw statements.
- Frontend source uses Next.js, MUI, TypeScript, SWR, and backend-loaded admin i18n.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + TypeScript/Next.js frontend
- **Package manager**: pnpm
- **Test framework**: Rust tests via `just test`; frontend validation via lint/build

## Deliverables

- Dashboard response types and SQL aggregates include cache hit rate, TTFB series, breakdown average latency, and activity base cost.
- Frontend dashboard cards and breakdown layout match the accepted plan.
- Admin seed i18n includes new dashboard labels.

## Done-When

- `just test` completes or any failure is documented.
- `pnpm lint:frontend` completes or any failure is documented.
- `pnpm build:frontend` completes or any failure is documented.
