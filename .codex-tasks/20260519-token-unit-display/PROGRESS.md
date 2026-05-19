# Progress

## 2026-05-19

- Started from the user request: token-related dashboard and performance monitoring values should display with K/M/B units.
- Initial search found primary frontend sites in `dashboard-kpi.tsx`, `dashboard-breakdown.tsx`, and `performance-monitoring-cards.tsx`.
- Added `fTokenCount` in `src/utils/format-number.ts` and applied it to dashboard token text plus performance monitoring LLM token text.
- Ran `pnpm --filter hook_frontend lint`; it passed.
