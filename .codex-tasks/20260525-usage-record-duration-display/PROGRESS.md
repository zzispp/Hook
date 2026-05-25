# Progress

## 2026-05-25

- Started from the user request: user usage records need the same duration text color and live scrolling timer behavior as admin request records, without row click drawer details.
- Implemented by exposing only `status` on user usage records, keeping `request_id` and upstream provider fields out of the user response.
- Reused the existing admin duration text renderer in the user usage table. User rows remain non-clickable and do not open a drawer.
- Validation passed: `pnpm --filter hook_frontend lint`, `pnpm --filter hook_frontend exec tsc --noEmit --pretty false`, `timeout 60 cargo test -p storage --test provider_request_records request_record_storage_lists_user_usage_records_without_upstream_fields`, and `timeout 60 cargo check -p types -p storage`.
