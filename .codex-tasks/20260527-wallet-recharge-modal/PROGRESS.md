# Progress Log

## Session Start

- **Date**: 2026-05-27 12:50 CST
- **Task name**: `20260527-wallet-recharge-modal`
- **Scope**: Move wallet recharge and card-code redemption from inline cards into dialogs; add real custom amount recharge order support because the requested modal contains an amount input.

## Completion Notes

- Backend create recharge order now accepts either `package_id` or `recharge_amount`; passing both or neither returns an explicit validation error.
- Custom amount orders write a real pending payment order with no package id, `Custom recharge` snapshot name, zero gift amount, system ratio payable amount, and existing payment channel/captcha flow.
- Wallet center no longer renders inline recharge/card-code cards. The breadcrumb action area now opens recharge and card-code dialogs.
- Recharge dialog shows ratio, payment channel/method, optional CAPTCHA, custom amount input, estimated payable, and package cards only when packages exist.
- Card-code redemption moved into a dialog; successful redemption closes it.
- Validation passed: `cargo fmt --all`, `timeout 60 cargo test -p types`, `timeout 60 cargo test -p recharge`, `timeout 60 cargo check -p backend`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.

## Hide User-Side Payment Channels

- Wallet recharge dialog now only renders payment method choices. Payment channel codes remain internal and are derived from the selected method before order creation.
- User recharge order rows no longer display payment channel name/code.
- Validation passed after hiding user-side channels: `timeout 60 cargo test -p recharge`, `pnpm lint:frontend`, `pnpm build:frontend`, `git diff --check`.
