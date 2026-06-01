# Billing Group Provider Key Scope

## Goal

Allow billing groups to restrict scheduling to specific provider API keys, while preserving provider-level allow semantics where no key restriction is configured.

## Boundary

- Backend group data model, storage, scheduling snapshot, proxy scheduler input, and validation.
- Admin UI group form/detail/table types and translations for selecting provider API keys.
- Tests first for scheduler behavior, then implementation and validation.
