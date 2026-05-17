# Progress Log

## 2026-05-17

- Started provider key endpoint format binding implementation.
- Added provider key `api_formats` and `allowed_model_ids` across DTOs, storage records, scheduling cache, baseline schema, and admin UI.
- Matched Aether semantics for key model permissions: empty `allowed_model_ids` means all models; non-empty list restricts scheduling by resolved global model id.
- Changed candidate routing to materialize endpoint/key route options only when the key supports the endpoint format and requested model.
- Migrated local PostgreSQL provider key data: added `api_formats` and `allowed_model_ids`, backfilled formats from provider endpoints, and initialized model permissions to `[]`.
- Validation passed: `cargo fmt --all --check`, `cargo check -p storage -p provider -p backend`, `cargo test -p backend llm_proxy::candidate -- --nocapture`, `cargo test -p provider -- --nocapture`, `cargo test -p storage -- --nocapture`, `pnpm lint:frontend`.
- Resumed for admin form semantics:
  - Limit visible API format options to OpenAI Chat/OpenAI CLI/OpenAI Compact/Claude Chat/Gemini CLI.
  - Source provider key model permissions from provider model bindings instead of global models.
  - Normalize endpoint base URL by trimming trailing `/` on both frontend and backend paths.
  - Add backend rejection for key model permissions outside the selected provider's model bindings.
- Implemented admin/provider alignment:
  - `ProviderApiKeyDialog` now loads `useProviderModels(providerId)` while open and builds permission options from bound `global_model_id` values.
  - `API_FORMAT_OPTIONS` is reduced to `openai_chat`, `openai_cli`, `openai_compact`, `claude_chat`, `gemini_cli`; key payload normalization drops legacy/unlisted formats on submit.
  - Endpoint create/update payloads call `normalizeBaseUrl`, and backend endpoint sanitization removes trailing `/` before validation/persistence.
  - Provider service validates `allowed_model_ids` against `list_model_bindings(provider_id)` before key create/update.
- Validation passed:
  - `pnpm --filter hook_frontend exec tsc --noEmit`
  - `pnpm lint:frontend`
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p provider validation -- --nocapture`
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p provider key_permissions -- --nocapture`
  - `cargo fmt --all --check`
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo check --workspace`
  - `pnpm build:frontend` passed; build printed an existing `Axios error: unauthorized` line while prerendering, but completed with exit code 0.
