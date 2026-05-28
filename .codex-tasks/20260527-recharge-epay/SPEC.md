# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Implement a channelized recharge payment system with payment abstractions in a dedicated `crates/payment` crate.
- Support provider config, payment order creation, epay notify verification, idempotent paid settlement, and frontend payment launch.

## Non-Goals

- Do not implement Stripe or Creedm providers.
- Do not add mock payment success paths or silent fallbacks.

## Constraints

- Follow repository `AGENTS.md` and `apps/hook_backend/AGENTS.md`.
- Preserve API key secrecy; never return or store it in plaintext.
- Keep failures explicit and typed.
- Backend changes use TDD where practical.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + Next.js frontend
- **Package manager**: pnpm
- **Test framework**: Rust unit tests, ESLint, Next build
- **Build command**: `just test`, `pnpm lint:frontend`, `pnpm build:frontend`

## Risk Assessment

- [x] External dependencies — epay protocol implemented from inspected new-api/go-epay behavior.
- [x] Breaking changes — recharge create-order response and payload intentionally change.
- [x] Long-running tests — Rust test wrapper uses repo 60-second timeout.

## Deliverables

- Payment provider abstraction and `epay` implementation under `crates/payment/src/channels/`.
- Storage/schema/type updates for payment channels and payment orders.
- Public epay notify API route and auth whitelist update.
- Wallet settlement for recharge and gift amounts.
- Admin channel config UI and user recharge payment launch UI.
- Backend/frontend validation.

## Done-When

- [ ] `epay` is registered disabled by default and configurable from admin settings.
- [ ] User recharge order creation returns form-post payment data.
- [ ] Epay success notify verifies signature and settles exactly once.
- [ ] Tests and checks listed in the final validation pass or failures are reported.

## Final Validation Command

```bash
just test && pnpm lint:frontend && pnpm build:frontend
```
