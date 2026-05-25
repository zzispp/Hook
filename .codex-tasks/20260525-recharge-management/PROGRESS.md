# Progress

- Started backend/domain implementation.
- Implemented backend recharge domain, storage entities, baseline tables, admin routes, RBAC API/menu seeds, and startup wiring.
- Extended system settings with recharge defaults, persistence, validation, and min/max bound checks.
- Implemented frontend recharge management tabs, package CRUD/status toggle UI, read-only order list, callback empty state, and recharge settings section.
- Added admin i18n seed copy for recharge navigation, tables, forms, status labels, empty states, and settings labels.
- Validation passed: `just check`, `cargo test -q -p recharge -p setting`, `pnpm lint:frontend`, `pnpm build:frontend`, and full `just test`.
