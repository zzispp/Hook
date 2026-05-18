# Progress

## 2026-05-18

- Started task after read-only audit found USD accounting mostly correct but wallet/card-code display still hardcoded to wallet currency.
- Tightened storage card-code redemption to reject non-USD accounting inputs and removed the cross-currency redemption conversion path.
- Validation: `cargo test -p storage card_code::redemption_currency` passed.
- User changed target: remove display-currency switching entirely. New target is fixed USD for both accounting and display.
- Updated SPEC/TODO to remove display-currency API/settings/frontend hooks instead of integrating them.
- Removed display-currency and exchange-rate backend APIs, state, config permissions, settings schema fields, and i18n seed leftovers.
- Fixed card-code generation and redemption to use the accounting/default wallet currency directly, with storage checks rejecting non-USD card/wallet/target currency inputs.
- Removed frontend display-currency hooks and API paths. Money formatting, labels, wallet/card-code/request-record/model pricing/API-token/performance displays now render fixed USD.
- Removed language-level currency selection from frontend number-format config; `fCurrency` always formats with USD.
- Validation passed:
  - `cargo fmt -q --all`
  - `git diff --check`
  - `/usr/bin/perl -e 'alarm shift; exec @ARGV' 60 cargo check -p backend -p setting -p card_code -p storage -p types -p currency`
  - `/usr/bin/perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage card_code::redemption_currency`
  - `/usr/bin/perl -e 'alarm shift; exec @ARGV' 60 just test`
  - `pnpm lint:frontend`
  - `pnpm build:frontend` passed; static generation logged `Axios error: connect ECONNREFUSED 127.0.0.1:5555` but exited 0.
  - Display-currency/exchange-rate residual keyword scan returned no matches.
- Second audit after user request:
  - Removed unused legacy i18n key `fields.costCny`.
  - Removed multi-currency select examples from `sections/_examples/mui/text-field-view` so demo routes do not present EUR/BTC/JPY currency switching.
  - Added storage-layer guardrails: request record and request candidate billing `cost_currency` now accepts only `None` or USD; non-USD returns `StorageError::Conflict`.
  - Added storage-layer card-code creation guard so non-USD card-code records fail before database insert.
  - Unified provider billing `ACCOUNTING_CURRENCY` with the shared `currency::ACCOUNTING_CURRENCY` constant.
  - Validation passed:
    - `cargo fmt -q --all`
    - `node -e` JSON parse for backend admin i18n seeds
    - `/usr/bin/perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage card_code::redemption_currency`
    - `/usr/bin/perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage --test provider_request_records`
    - `/usr/bin/perl -e 'alarm shift; exec @ARGV' 60 cargo test -p storage --test provider_request_candidates`
    - `/usr/bin/perl -e 'alarm shift; exec @ARGV' 60 cargo check -p backend -p provider -p storage -p card_code -p currency`
    - `/usr/bin/perl -e 'alarm shift; exec @ARGV' 60 just test`
    - `pnpm lint:frontend`
    - `pnpm build:frontend` passed; static generation again logged `Axios error: connect ECONNREFUSED 127.0.0.1:5555` but exited 0.
  - Final audit scan: CNY appears only in tests that assert non-USD is rejected; display-currency/exchange-rate/hook/API/old-key scans returned no matches.
