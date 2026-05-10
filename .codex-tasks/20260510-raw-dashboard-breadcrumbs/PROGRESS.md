# Progress

## 2026-05-10

- Current dashboard nav no longer translates menu values, but page breadcrumbs still use `t('pages.*')` and `t('nav.*')`.
- Need a shared resolver that reads current section and item from `useNavbar()` so titles/breadcrumbs follow DB values.
- Added `useDashboardBreadcrumbs()` and routed admin/user dashboard breadcrumb consumers through navbar-derived raw values.
- Added `dashboard-menu-values.ts` for baseline raw fallback values matching migration defaults; this prevents first-render fallback from using frontend locale keys.
- Updated relevant dashboard metadata titles to use the same raw baseline values.
- Validation passed: `pnpm --filter hook_frontend exec tsc --noEmit --pretty false`.
- Search validation found no remaining `heading={t(...)}`, `heading: t(...)`, `section: t('nav.*')`, or `pages.*` fallbacks in the admin/token/model/wallet dashboard sections.
