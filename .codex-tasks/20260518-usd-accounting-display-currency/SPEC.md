# Fixed USD Accounting And Display

## Goal

All ledger/accounting values and all display surfaces use USD. Display currency configuration, exchange-rate display conversion, and currency switching are removed.

## Scope

- Keep writable amount inputs clearly labeled as USD because submitted amounts are accounting values.
- Remove the system display-currency setting, API, RBAC entries, and frontend hooks/types.
- Remove USD/CNY display conversion and card-code cross-currency redemption dependencies.
- Keep wallet/card-code/provider/request-record amount data in USD and surface non-USD data as explicit errors at storage boundaries where applicable.

## Validation

- Backend targeted tests for storage/card-code currency boundary.
- Frontend lint/build or focused TypeScript validation where feasible.
