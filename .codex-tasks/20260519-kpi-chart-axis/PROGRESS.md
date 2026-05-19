# Progress

- Located `DashboardKpiCard` in `apps/hook_frontend/src/sections/overview/analytics/view/dashboard-kpi.tsx`.
- Cause: KPI sparkline uses an 84x56 ApexCharts SVG while default y-axis labels remain enabled; labels are positioned left of the plot and clipped by the SVG viewport.
- Fixed the KPI chart options by disabling x/y axis labels for the sparkline card only.
- Validation passed: `pnpm --filter hook_frontend lint`.
