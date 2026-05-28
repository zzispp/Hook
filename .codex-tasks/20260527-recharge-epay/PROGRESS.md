# Progress Log

---

## Session Start

- **Date**: 2026-05-27 10:46
- **Task name**: `20260527-recharge-epay`
- **Task dir**: `.codex-tasks/20260527-recharge-epay/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv
- **Environment**: Rust workspace / Next.js / cargo + pnpm

---

## Context Recovery Block

- **Current milestone**: #6 — Run final validation and summarize
- **Current status**: DONE
- **Last completed**: #6
- **Current artifact**: `TODO.csv`
- **Key context**: Implementing recharge payment channel abstraction in a new `crates/payment` crate; `epay` lives under `payment/src/channels/`.
- **Known issues**: `just test` still fails in an unrelated LLM proxy test: `apps/hook_backend/src/llm_proxy/formats.rs::streaming_requests_do_not_route_to_force_non_stream_formats`.
- **Next action**: Summarize implementation and validation results to the user.

## Completion Notes

- Added `crates/payment` with provider abstractions, registry, and `channels/epay.rs`.
- Wired recharge order creation, epay notify verification, encrypted secret handling, and idempotent settlement through recharge/storage ports.
- Added public `/api/payment-channels` for user recharge selection and `/api/payment/{code}/notify` for epay callbacks.
- Added admin channel configuration UI and wallet recharge payment form submission.
- Validation passed: `cargo test -p payment`, `cargo test -p recharge`, `cargo test -p storage`, `cargo test -p setting`, `cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`.
- Validation blocked outside this feature: `just test` fails in `llm_proxy::formats::tests::streaming_requests_do_not_route_to_force_non_stream_formats`; `cargo clippy -p backend --all-targets -- -D warnings` is blocked by existing `crates/formats` collapsible-if warnings.

## Follow-up Notes

- Added shared `public_base_url` regex validation in `crates/types`; it requires `http://` or `https://` and rejects localhost/private network hosts.
- System settings now reject invalid public base URLs when saved.
- Enabling any payment channel now requires a non-empty valid HTTP/HTTPS `public_base_url`; disabled channels can still be saved without it.
- Validation passed after follow-up: `cargo fmt --all`, `timeout 60 cargo test -p types`, `timeout 60 cargo test -p setting`, `timeout 60 cargo test -p payment`, `timeout 60 cargo test -p recharge`, `timeout 60 cargo test -p storage`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`.
- Removed the stale wallet custom-amount placeholder panel; the user wallet recharge entry now opens directly to package recharge with payment channel/method selection.
- Validation passed after wallet UI follow-up: `pnpm lint:frontend`, `pnpm build:frontend`.
- Added an explicit recharge entry button inside Wallet Center that scrolls to the in-page recharge/payment section.
- Validation passed after recharge entry follow-up: `pnpm lint:frontend`, `pnpm build:frontend`.

## Recharge CAPTCHA Follow-up

- Added `recharge_captcha_enabled` to system settings types, storage mapping, baseline migration seed, captcha config API, and backend i18n seed.
- Extended `CaptchaUseCase` with `verify_recharge`; create recharge order now verifies and consumes the redeemed CAPTCHA token before creating a payment order when the switch is enabled.
- Added captcha behavior tests for recharge config, disabled mode, missing token, and one-time token consumption.
- Added the admin recharge settings switch and wallet recharge CAPTCHA widget; create order payload now sends `captcha_token` when required.
- Validation passed after recharge CAPTCHA follow-up: `cargo fmt --all`, `timeout 60 cargo test -p types`, `timeout 60 cargo test -p setting`, `timeout 60 cargo test -p captcha`, `timeout 60 cargo test -p recharge`, `timeout 60 cargo test -p user`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Recharge Enablement Guard Follow-up

- Wallet Center now gates the recharge button and dialog by `recharge_enabled`, so user-side recharge is hidden when the backend switch is off.
- Setting service now depends on an injected payment-channel catalog and rejects `recharge_enabled=true` unless at least one channel is enabled and has a saved secret.
- Startup wires the setting service to `RechargeStore` for payment channel readiness checks.
- Added setting unit tests for rejecting and allowing recharge enablement based on payment channel readiness.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo test -p setting`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Payment Channel Public URL Frontend Guard

- Payment channel save now checks the saved system `public_base_url` before allowing an enabled channel PATCH.
- Empty saved public base URL shows a translated frontend toast instead of surfacing the backend English error.
- Invalid saved HTTP/HTTPS URL shows a separate translated frontend toast.
- Added backend i18n seed keys for CN/EN admin resources.
- Validation passed after this follow-up: `cargo fmt --all`, `pnpm lint:frontend`, `pnpm build:frontend`, `timeout 60 cargo check -p backend`, `git diff --check`.

## Wallet CAPTCHA Label Localization

- `AuthCaptcha` now accepts explicit label text so non-auth pages do not depend on the auth namespace being loaded.
- Wallet recharge modal passes admin namespace CAPTCHA labels to the shared CAPTCHA widget.
- Added admin CN/EN i18n seed keys for CAPTCHA widget labels.
- Validation passed after this follow-up: `cargo fmt --all`, `pnpm lint:frontend`, `pnpm build:frontend`, `timeout 60 cargo check -p backend`, `git diff --check`.

## Payment Window And Polling Follow-up

- Wallet recharge now opens an `about:blank` payment window synchronously before creating an order, then submits the provider form into that window after order creation.
- If the browser blocks the payment window, the frontend shows an admin i18n message and does not create an order.
- After opening payment, the wallet page refreshes user recharge orders and wallet state on a short interval until the order is paid or the polling budget ends.
- Added `poll_pending_payment_orders` to the recharge use case and repository port.
- Added storage query for unexpired pending recharge orders and service logic to call the registered provider query API, settling paid orders through the existing idempotent transaction path.
- Added `recharge_payment_poll` scheduled task with configurable scan limit and admin i18n labels.
- Epay query now uses the common `api.php?act=order&pid=...&key=...&out_trade_no=...` protocol and maps successful trade status to paid; refund remains explicitly unsupported.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo test -p payment`, `timeout 60 cargo test -p recharge`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Recharge Ready-Channel Frontend Guard

- System settings save now checks the loaded payment channel list before submitting `recharge_enabled=true`.
- Recharge enablement is blocked in the frontend unless at least one channel is enabled and has `secret_set=true`.
- Loading, unavailable, and missing-ready-channel states use backend-seeded admin i18n keys instead of surfacing raw backend errors.
- `RechargeSettingsSection` now receives the page-level payment channel resource so the settings page does not issue a duplicate channel request.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## System Settings Frontend Validation Follow-up

- Added a unified system settings frontend validation module that runs before `PATCH /admin/settings/system`.
- The frontend now mirrors backend-known constraints for site fields, public base URL, default user group, numeric limits, request-record header names, recharge bounds, SMTP fields, email suffix rules, email templates, and email-feature prerequisites.
- The payment channel public URL guard now reuses the same public-base-URL validator used by system settings.
- Added backend-seeded CN/EN admin i18n messages for the new validation toasts.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Wallet Payment Popup Navigation Fix

- Replaced the `about:blank` payment popup flow with a named writable popup and a current-page hidden form targeting that popup.
- The popup now shows translated creating/redirecting status text while the order is created and the provider POST is submitted.
- This keeps the payment POST tied to the pre-opened user-gesture window instead of relying on a dynamic form submit inside `about:blank`.
- Validation passed after this follow-up: `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Unified Payment Channel Save Follow-up

- Removed the per-channel save action from the system settings recharge section; payment channel fields are now controlled by the page form state.
- The top-level system settings save button persists changed payment channel config through the shared save coordinator.
- When channel config changes, the coordinator validates channel prerequisites before sending any request, then saves the public base URL/system settings, channel config, and final recharge switch state in order.
- Validation passed after this follow-up: `pnpm lint:frontend`, `pnpm build:frontend`, `timeout 60 cargo check -p backend`, `git diff --check`.

## Payment Callback Endpoint And Return Settlement Follow-up

- Payment providers now declare public callback endpoints through the payment abstraction; epay declares both notify and return endpoints.
- Backend authorization merges config whitelist entries with registered payment callback endpoints at startup, so payment callback whitelist paths no longer live in `config/config.yaml`.
- Recharge order creation now builds notify and return URLs from the registered provider endpoint declarations instead of hard-coded channel paths.
- Added `/api/payment/{code}/return` handling that verifies the provider return payload, settles the order through the same callback path, and redirects back to `/dashboard/wallet`.
- Wallet recharge now warns users that payment opens in a new tab, submits the payment form inside that tab, and stops order polling when the recharge modal closes.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo test -p payment`, `timeout 60 cargo test -p recharge`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Trailing Slash Payment Callback Follow-up

- Added trailing-slash variants for the generic payment notify and return routes, so provider URLs such as `/api/payment/epay/return/` no longer 404.
- Dynamic payment callback whitelist generation now derives both `/api/.../return` and `/api/.../return/` from registered provider endpoints; no concrete channel path is hard-coded into config or auth rules.
- Added route and auth tests covering the trailing-slash return URL.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo test -p recharge trailing_slash_payment_return_route_redirects_to_wallet`, `timeout 60 cargo test -p backend payment_callback_rule_includes_trailing_slash_variant`, `timeout 60 cargo test -p recharge`, `timeout 60 cargo check -p backend`, `git diff --check`.

## Wallet Payment Pending State Follow-up

- Removed the pre-submit new-tab warning from the recharge dialog.
- After a recharge order is created and the payment form is submitted to a new tab, the dialog now switches to a pending payment state that says the user should complete payment in the new tab.
- Added an explicit `I have paid` action that refreshes the current recharge order list and wallet balance once, then leaves the dialog pending or clears it when the order is paid.
- Split wallet payment window, polling, dialog form content, pending panel, and shared props into small files under the frontend wallet section.
- Validation passed after this follow-up: `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Scheduled Task List Registration Follow-up

- Scheduler task listing now synchronizes missing registered task records before reading the `scheduled_tasks` table.
- This makes registered runtime tasks such as `recharge_payment_poll` visible in the admin scheduled task list even when the database table does not yet contain that row.
- Added a scheduler query test covering missing registry task insertion during list reads.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo test -p scheduler`, `timeout 60 cargo check -p backend`, `git diff --check`.

## Max Unpaid Orders And Payment Safety Follow-up

- Added `recharge_max_unpaid_orders` to system settings, storage mapping, baseline seed, backend i18n seed, frontend admin settings form, and frontend validation. Default value is `5`.
- Create recharge order now passes the configured limit into storage. Storage creates the order inside one transaction, locks the user row, counts this user's unexpired pending orders, and rejects creation when the count reaches the configured limit.
- Wallet recharge maps the unpaid-order-limit conflict to a translated user-facing message.
- Payment query abstraction now returns status plus provider trade number, payment method, amount, and raw payload. Epay query maps `trade_no`, `type`, and `money` into that result.
- Settlement now verifies payment channel and provider amount, requires a non-empty provider trade number, and rejects reused provider trade numbers before any wallet write.
- Added a partial unique baseline index for `(payment_channel_code, provider_trade_no)` when provider trade number is present.
- Added recharge service tests for unpaid-order limit and polling settlement safety, plus storage mock tests for transaction order, rollback without wallet writes, missing provider trade number, missing amount, and reused provider trade number.
- Split recharge service tests into smaller order creation and payment polling modules to keep files within the project size rule.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo test -p payment`, `timeout 60 cargo test -p recharge`, `timeout 60 cargo test -p setting`, `timeout 60 cargo test -p storage`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Frontend API Proxy Follow-up

- Confirmed the ngrok 404 response came from the Next.js frontend dev server rather than the Rust backend.
- Added a Next.js rewrite for `/api/:path*` to the same `HOOK_BACKEND_URL` backend target already used by `/v1` and `/v1beta`.
- This lets ngrok remain pointed at the frontend port while payment `return` and `notify` callbacks still reach backend routes.
- Validation passed after this follow-up: `pnpm --filter hook_frontend lint`, `pnpm --filter hook_frontend build`.

## Payment Callback Records Follow-up

- Real provider notify/return requests are now persisted in `payment_callback_records` before verification and updated after processing.
- Successful paid callbacks are marked `processed` with order number, provider trade number, payment method, trade status, raw params, and settlement state.
- Verification or settlement failures are marked `failed` and keep the raw provider params plus the explicit error message for admin diagnosis.
- The admin recharge callback tab now calls `/api/admin/payment-callbacks` and renders a paginated/filterable callback table instead of the old placeholder copy.
- Backend admin i18n seeds were updated so fresh baselines no longer say callbacks are not implemented.
- Validation passed after this follow-up: `cargo fmt --all`, `timeout 60 cargo test -p recharge`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.
