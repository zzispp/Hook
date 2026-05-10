# API Binding Transfer List

## Goal

Replace the menu management API binding checkbox modal with a clearer transfer list so admins can move API permissions between unbound and bound columns.

## Scope

- Update `menu-api-binding-dialog.tsx`.
- Preserve existing selected API ids state contract.
- Keep method labels, translated API names, and path details visible.
- Validate with frontend lint.

## Validation

- `pnpm lint:frontend`
