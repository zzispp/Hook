# Recharge Package Display Fix

## Goal

Remove the user recharge note "Buying a package creates a pending order..." from user-facing copy, show the backend recharge ratio as a separate warning, and fix the user-side package list so active packages created in admin appear instead of the empty state.

## Scope

- User wallet recharge UI.
- Admin i18n seed strings that provide user-facing recharge copy.
- Recharge package query path across frontend and backend where needed.

## Validation

- Inspect code paths for settings, recharge packages, and translations.
- Run focused frontend lint/build or TypeScript checks when feasible.
- Run Rust checks/tests when backend behavior changes.
