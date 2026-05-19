# Progress

- Identified backend parser-only expiry validation in crates/api_token/src/application/validation.rs.
- Added backend future-time validation for API token expires_at.
- Added frontend datetime min and updated backend i18n seed helper text.
- Targeted Rust test command is currently blocked by existing storage user repository compile error: missing sea_orm::Condition import.
- Retried targeted Rust tests; cargo test -p api_token validation_tests -- --nocapture passed with 5 tests.
