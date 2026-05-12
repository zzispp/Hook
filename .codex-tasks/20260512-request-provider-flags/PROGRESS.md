# Progress

## 2026-05-12

- Verified Aether request table behavior: `has_fallback` is true when executed request candidates contain more than one distinct `candidate_index`; `has_retry` is true when any executed candidate has `retry_index > 0`. The frontend gives failover icon priority over retry icon.
- Added `provider_key_name`, `provider_key_preview`, `has_failover`, and `has_retry` to request record aggregates. The flags are derived from executed `request_candidates` only, matching Aether semantics.
- Updated the admin request records provider column to render provider name, provider key sublabel, and the Aether-style failover/retry icons with backend-seeded i18n tooltips.
- Validation passed: `cargo test -p storage --test provider_request_records -- --nocapture`, `cargo check -p backend`, targeted frontend ESLint, `pnpm lint:frontend`, and `pnpm build:frontend`. The build exited successfully while printing an existing `Axios error: unauthorized` during static generation.
