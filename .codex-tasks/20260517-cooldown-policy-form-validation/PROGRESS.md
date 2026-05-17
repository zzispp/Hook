# Progress

## 2026-05-17

- Confirmed backend validation for `provider_cooldown_policy`:
  - Empty `rules` means disabled and is valid.
  - Non-empty `rules` require positive `window_seconds`.
  - Rule `status_code` must be 100-599.
  - Rule `failure_count` and `cooldown_seconds` must be positive.
  - Duplicate `status_code` is invalid.
- Next: align frontend dialog display and save-time validation.
- Implemented frontend dialog behavior:
  - Disabled/no-rule policy hides the fixed window field.
  - Clicking add rule displays fixed window and a rule row.
  - Saving no rules submits `{ window_seconds: 0, rules: [] }`.
  - Saving with rules requires positive fixed window seconds.
  - Added rules must be fully filled before save.
  - Status code must be an integer between 100 and 599.
  - Failure count and cooldown seconds must be positive integers.
  - Duplicate status codes are rejected before API submission.
- Added backend admin i18n seed messages for CN/EN validation copy.
- Validation passed:
  - `pnpm lint:frontend`
  - `pnpm build:frontend`
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo check --workspace`
