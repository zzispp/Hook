# Progress Log

## 2026-05-17

- Started provider key endpoint format binding implementation.
- Added provider key `api_formats` and `allowed_model_ids` across DTOs, storage records, scheduling cache, baseline schema, and admin UI.
- Matched Aether semantics for key model permissions: empty `allowed_model_ids` means all models; non-empty list restricts scheduling by resolved global model id.
- Changed candidate routing to materialize endpoint/key route options only when the key supports the endpoint format and requested model.
- Migrated local PostgreSQL provider key data: added `api_formats` and `allowed_model_ids`, backfilled formats from provider endpoints, and initialized model permissions to `[]`.
- Validation passed: `cargo fmt --all --check`, `cargo check -p storage -p provider -p backend`, `cargo test -p backend llm_proxy::candidate -- --nocapture`, `cargo test -p provider -- --nocapture`, `cargo test -p storage -- --nocapture`, `pnpm lint:frontend`.
