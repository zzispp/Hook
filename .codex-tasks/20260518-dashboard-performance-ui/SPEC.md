# Dashboard And Performance UI Refresh

## Goal
Update the real-data dashboard and performance monitoring pages to reuse suitable Minimal chart templates without adding mock data or changing backend contracts.

## Scope
- Dashboard trend chart uses the Analytics Website Visits visual style.
- Dashboard breakdown/ranking cards use visually richer ranking/distribution templates where data maps directly.
- Performance monitoring model/provider distribution cards use the same distribution visual language.
- Performance monitoring range selector matches dashboard segmented controls.

## Validation
- `pnpm lint:frontend`
- `pnpm build:frontend`
