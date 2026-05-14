# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Audit every business money path for bugs caused by switching the system display currency.
- Separate accounting/storage currency from display currency where the current implementation mixes them.
- Add a reusable backend currency conversion boundary if the audit shows repeated conversion logic.

## Non-Goals

- Do not change business pricing semantics without evidence.
- Do not add mock or silent fallback currency behavior.
- Do not localize frontend through locale JSON files.

## Constraints

- Accounting and counters must remain deterministic across display-currency changes.
- Request pricing/model pricing stays in its accounting currency unless code evidence shows otherwise.
- Conversion failures must surface explicitly.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust 2024, TypeScript/Next.js
- **Package manager**: pnpm
- **Test framework**: cargo tests, frontend lint/build

## Deliverables

- Code audit notes in `PROGRESS.md`.
- Shared currency conversion module when useful.
- Fixes for confirmed currency-switching risk points.
- Focused tests and validation commands.

## Done-When

- [ ] All real monetary domains are classified as accounting, snapshot, or display-only.
- [ ] Confirmed risks are fixed or explicitly documented as no-change with evidence.
- [ ] Validation passes.

## Final Validation Command

```bash
cargo test -q -p currency && cargo test -q -p card_code && cargo test -q -p storage card_code::redemption_currency && cargo check -q && pnpm lint:frontend && pnpm build:frontend && git diff --check
```
