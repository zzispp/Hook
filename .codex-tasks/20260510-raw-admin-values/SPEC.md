# Raw Values In Admin Configuration Screens

## Goal

Admin configuration screens should display API names and menu titles as stored values, not translated labels. Runtime navigation can still translate user-facing nav labels.

## Scope

- Inspect all usages of admin translation helpers for API/menu/section names.
- Replace translated API names and menu titles in admin configuration tables/dialogs with raw `name` / `title` values where those fields are value/configuration data.
- Keep dashboard navigation translation intact.
- Validate frontend lint.

## Validation

- `pnpm lint:frontend`
