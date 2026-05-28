# Aether User Stats

## Task Shape

- **Shape**: `single-full`

## Goals

- Add an admin-only overview module matching Aether `/admin/user-stats`.
- Provide user leaderboard, selected user summary, selected user cost trend, and optional comparison trend.
- Serve Aether-shaped admin stats endpoints from pre-aggregated user usage buckets.

## Non-Goals

- No migration files; development baseline is the schema source.
- No pixel-perfect Vue/Tailwind visual clone.
- No silent fallback or mocked success path.

## Constraints

- Use backend-controlled admin i18n seed JSON.
- Query high-volume dashboard data from aggregate buckets, not full request-record scans.
- Keep failures explicit.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + Next.js frontend
- **Package manager**: pnpm
- **Test framework**: cargo tests, frontend lint/build
- **Build command**: `just test && pnpm lint:frontend && pnpm build:frontend`

## Deliverables

- Baseline aggregate table and indices.
- Request-record bucket upsert path.
- Admin stats endpoints:
  - `GET /api/admin/stats/leaderboard/users`
  - `GET /api/admin/usage/stats`
  - `GET /api/admin/stats/time-series`
- Admin overview user stats section and translations.

## Done-When

- [ ] New endpoints return Aether-shaped data.
- [ ] New records update aggregate buckets.
- [ ] Admin overview renders the user stats module.
- [ ] Validation commands complete or any failure is documented with concrete cause.

## Final Validation Command

```bash
just test && pnpm lint:frontend && pnpm build:frontend
```
