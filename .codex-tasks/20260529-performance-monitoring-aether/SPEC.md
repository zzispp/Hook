# Performance Monitoring Aether Refactor

## Goal

Implement the approved Aether-style performance monitoring refactor across backend data/API and frontend UI.

## Constraints

- Development-stage destructive changes are allowed.
- Keep baseline clean; do not add online migrations or compatibility readers for old snapshot JSON.
- Do not introduce mock data, silent fallbacks, or fake success paths.
- Keep admin i18n backend-controlled through seed JSON.

## Deliverables

- Updated performance snapshot metrics without business-only visible KPIs.
- New analytics API with percentiles, errors, upstream performance, and recent errors.
- Real provider cooldown event source for circuit breaker/cooldown counts.
- Aether-style frontend layout and charts/tables.
- Automated validation through Rust and frontend checks where feasible.
