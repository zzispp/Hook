# Cooldown Policy Form Validation

## Goal

Make the provider cooldown policy dialog match backend semantics before save.

## Requirements

- Default disabled policy should not show the fixed window field.
- Clicking add rule shows the fixed window field and a rule row.
- Any added rule must be fully filled before save.
- When rules exist, fixed window seconds is required and must be positive.
- Status code must be 100-599.
- Failure count and cooldown seconds must be positive.
- Duplicate status codes are rejected before save.
- No-rule payload should save as disabled cooldown policy: `{ window_seconds: 0, rules: [] }`.

## Validation

- Run frontend lint.
- Run frontend build.
