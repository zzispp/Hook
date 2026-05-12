# Progress

## 2026-05-12

- Started by reading Hook request records view, active records storage query, and aether Usage.vue active polling.
- Changed Hook active records query so empty-id discovery still returns active rows only, while polling known ids returns their latest aggregated state including terminal success/failed.
- Updated frontend request records view to keep global refresh and active polling separate, pause both when the page is hidden, and derive the selected drawer record from the latest list row.
- Updated the detail drawer to prefer the freshest status-ranked record between list polling data and detail fetch data.
- Added a storage regression assertion that polling known active ids returns terminal success records as well as still-active streaming records.
- Validation passed: cargo check -p backend, pnpm --filter hook_frontend build, git diff --check, and perl 60s cargo test -p storage --test provider_request_records -- --nocapture.
