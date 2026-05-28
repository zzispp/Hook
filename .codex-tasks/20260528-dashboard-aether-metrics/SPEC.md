# Dashboard Aether Metrics

## Goal

Update Hook dashboard to show the requested Aether-style metric groups: today request/token/cost/user cards, monthly health summary, 7-day statistics controls, cost distribution charts, daily statistics table, and 7-day totals.

## Constraints

- Preserve backend-controlled admin i18n.
- Do not add mock success or silent fallback data paths.
- Use current repository patterns for dashboard API, types, actions, and UI.
- Validate frontend changes with lint/build when feasible.

## Evidence Needed

- Current Hook dashboard frontend and backend data contract.
- Aether dashboard metrics and source data structure.
- Read sites for any changed dashboard request parameters or response fields.
