# Context Notes

- Hook stores global settings in the single `system_settings` row, surfaced through `crates/types/src/system_setting.rs`, `crates/setting`, and `crates/storage/src/setting`.
- Admin system settings UI lives in `apps/hook_frontend/src/sections/admin/system-settings-*.ts(x)` and reads backend-seeded admin i18n from `apps/hook_backend/src/migration/defaults/i18n/admin.*.json`.
- Request record details are stored on `request_candidates.request_headers`, `request_body`, and `response_body`.
- Current capture redacts a hardcoded header list in `apps/hook_backend/src/llm_proxy/proxy/capture.rs`.
- Current request body capture is written when available candidate records are inserted; response body capture is written on attempt update.
- Aether reference has `request_record_level`, max body sizes, and `sensitive_headers`; user requested adding independent switches for request headers, request body, and response body.
- Semantics for new settings: `request_record_level` is the persisted detail-level selector, `record_request_headers` gates request header storage/redaction, `record_request_body` gates request body storage/truncation, and `record_response_body` gates response body storage/truncation.
