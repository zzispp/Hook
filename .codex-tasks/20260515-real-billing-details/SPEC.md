# Real Billing Details Verification

## Goal

Verify with real upstream providers and the local database that request records persist billing detail fields and service tier for non-stream chat completions.

## Boundary

- Use provider secrets only through environment variables.
- Seed only deterministic local test fixtures.
- Send a real `/v1/chat/completions` request through the local backend.
- Verify `request_records` and `request_candidates` values directly from Postgres.

## Validation

- `cargo fmt --check`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo check -p backend`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p provider billing`
- `perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage --test provider_request_records -- --nocapture`
- `node .codex-tasks/20260515-real-billing-details/real_billing_details.mjs`
