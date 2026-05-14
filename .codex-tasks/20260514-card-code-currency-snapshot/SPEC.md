# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Add a currency snapshot to generated card codes based on the current system display currency.
- Display card code amounts using the card code snapshot currency.
- Redeem card codes into the current system currency, converting through the existing USD/CNY rate when the snapshot currency differs.

## Non-Goals

- Do not add a separate card code usage-status column.
- Do not add frontend locale JSON files.
- Do not add fallback/mock exchange behavior.

## Constraints

- Follow existing Rust workspace, SeaORM, Next.js, and admin i18n seed patterns.
- Keep failures explicit when currency or exchange conversion cannot be resolved.
- Use existing system settings and exchange-rate infrastructure.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024, TypeScript/Next.js
- **Package manager**: pnpm
- **Test framework**: cargo tests, frontend lint/build

## Risk Assessment

- [x] External dependencies (APIs, services) — existing exchange cache path identified.
- [x] Breaking changes to existing code — card code DB model and wallet redemption path affected.
- [x] Large file generation — not applicable.
- [x] Long-running tests — use repository commands and 60s backend test expectation.

## Deliverables

- Backend schema/types/entities/repository changes for card code currency.
- Backend generation and redemption logic using system currency and exchange-rate conversion.
- Frontend card code amount display/export carrying snapshot currency.
- Focused tests for generation snapshot and redemption conversion.

## Done-When

- [ ] New card codes persist the system currency snapshot.
- [ ] Card code table/export display snapshot currency.
- [ ] Redemption credits wallet in current system currency and converts when needed.
- [ ] Backend and frontend validation pass or known existing failures are documented.

## Final Validation Command

```bash
cargo test -q -p card_code && cargo check -q && pnpm lint:frontend && pnpm build:frontend && git diff --check
```
