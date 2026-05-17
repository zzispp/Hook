# Billing And Monitoring Zero Diagnostics

## Goal

Find why request record billing displays as `0` and why performance monitoring core/LLM metrics display all zeros.

## Scope

- Read local PostgreSQL request records, request candidates, billing snapshots, billing rules, dimension collectors, usage and performance monitoring snapshot tables.
- Trace backend write path for billing and performance monitoring aggregation.
- Fix root cause if code or seed data is wrong.
- Validate with direct DB queries and automated checks.

## Constraints

- Use real local DB data, not mocks.
- Do not hide billing failures by silently writing `0`.
- Preserve existing unrelated changes.
