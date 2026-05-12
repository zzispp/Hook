# Progress

- Started: 2026-05-12
- Status: Done
- Scope: Make admin token table owner cells readable by username or email.

## Completed

- Created task tracking files.
- Confirmed the admin token table renders `row.user_id` directly.
- Confirmed `ApiTokenResponse` currently carries only `user_id`, not username or email.
- Confirmed the existing frontend user option load is capped at 100 users, so client-side owner lookup would be incomplete.
- Added `owner` data to token responses and enrich admin token lists from the user catalog.
- Added storage user batch lookup for owner identity enrichment.
- Updated the token table owner cell to display username and email, not the raw user id.
- Added an application test for admin token owner enrichment.
- Added configured system admin as an explicit owner source for API-token owner enrichment.
- Extended `AdminFiltersToolbar` with a composable extra-control slot.
- Added token management filters for search, status, and admin token type.
- Added `filters.allTokenTypes` to backend admin i18n seeds.

## Current

- Validation for the system-admin owner fix is blocked by an unrelated compile error in `crates/storage/src/provider/request_record_query.rs`.

## Validation

- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p api_token admin_token_list_includes_owner_identity`
- `cargo check -p api_token`
- `pnpm lint:frontend`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p api_token`
- `cargo check -p backend`
- `pnpm build:frontend`
- `git diff --check`
- `rg -n "row\\.user_id|id: 'user_id'|fontFamily: 'monospace'.*user_id|user_id \\?\\?" apps/hook_frontend/src/sections/api-tokens/api-token-table.tsx`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p api_token system_owner` failed before reaching api_token due to `RequestRecordDetail` missing `request_headers` and `response_body` in `crates/storage/src/provider/request_record_query.rs`.
- `cargo check -p backend` failed on the same unrelated storage compile error.
- `cargo check -p api_token` failed on the same unrelated storage compile error.
- `git diff --check -- apps/hook_backend/src/startup.rs crates/api_token/src/infra/storage_repository.rs .codex-tasks/20260512-token-owner-display/TODO.csv .codex-tasks/20260512-token-owner-display/PROGRESS.md`
- `pnpm lint:frontend`
- `pnpm build:frontend` passed; Next still prints the existing `Axios error: unauthorized` during static page generation, but exits 0.
