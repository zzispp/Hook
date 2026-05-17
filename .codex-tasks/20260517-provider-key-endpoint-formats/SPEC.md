# Provider Key Endpoint Formats

Goal: Provider keys declare supported endpoint formats and scheduling only combines keys with compatible endpoints.

## 2026-05-17 Scope

- Key model permission options must come from the selected provider's model binding list, not the global model catalog.
- Endpoint/API format selectors are limited to the operational formats currently exposed in admin: `openai_chat`, `openai_cli`, `openai_compact`, `claude_chat`, `gemini_cli`.
- Provider endpoint `base_url` must be persisted without trailing `/` characters, both from the frontend payload and backend sanitization.
- Backend key create/update must reject `allowed_model_ids` that are not bound to the target provider.
