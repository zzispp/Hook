# Progress

- Created task tracking files.
- Implemented backend system settings fields:
  - support ticket captcha switch, default enabled.
  - token quantity limit, default 5.
  - token expiry check interval in minutes, default 5.
- Wired support ticket captcha verification through the captcha use case and operations service.
- Wired token quantity limit into user/admin token creation and token expiry cleanup interval into expired-token cleanup.
- Implemented frontend admin settings controls and ticket-create captcha token submission.
- Added focused api_token test coverage for owner token limit and cleanup interval behavior.

## Validation

- `cargo fmt --all`: passed.
- `cargo test -p api_token cleanup_expired_tokens_skips_second_run_inside_interval`: passed.
- `cargo check -p api_token -p backend`: passed.
- `pnpm lint:frontend`: passed.
- `just check`: passed.
- `pnpm build:frontend`: passed.
- `just test`: passed.
- Re-ran after frontend captcha config state correction:
  - `cargo fmt --all`: passed.
  - `cargo check -p api_token -p operations -p captcha -p setting -p storage -p user -p backend`: passed.
  - `cargo test -p api_token cleanup_expired_tokens_skips_second_run_inside_interval`: passed.
  - `pnpm lint:frontend`: passed.
  - `pnpm build:frontend`: passed.
  - `just test`: passed.
