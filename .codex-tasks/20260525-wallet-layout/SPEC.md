# Wallet Center Layout

## Goal

Implement the approved wallet center layout: balance summary first, recharge and card-code entry as the deposit operation layer, and ledger as the full-width audit layer.

## Scope

- Frontend wallet center layout and component extraction.
- Wallet recharge panel structure for future arbitrary amount recharge and current package recharge.
- Admin i18n seed keys used by the new wallet layout.

## Non-Goals

- Do not implement real arbitrary amount recharge backend flow.
- Do not introduce mock successful payment or silent fallback behavior.

## Validation

- Frontend lint.
- i18n JSON parse.
- Keep touched frontend files below 300 lines and functions under local limits.
