# Auth and Main Header Language Popover

## Goal

Add the existing language popover icon button to public main headers and auth headers so sign-in, sign-up, forgot-password, reset-password, and the home page can switch languages.

## Scope

- Reuse the current `LanguagePopover` and `allLangs` i18n state.
- Update only the header layout components that render these pages.
- Validate with frontend lint/build checks where feasible.

## Constraints

- Do not add frontend locale JSON fallback resources.
- Do not introduce fallback language behavior.
- Keep UI consistent with existing header action spacing.
