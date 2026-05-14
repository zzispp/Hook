# Progress Log

---

## Session Start

- **Date**: 2026-05-14 15:12 CST
- **Task name**: `20260514-card-code-currency-snapshot`
- **Task dir**: `.codex-tasks/20260514-card-code-currency-snapshot/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv
- **Environment**: Rust workspace / Next.js / cargo + pnpm

---

## Context Recovery Block

- **Current milestone**: #5 — Commit amend and force push
- **Current status**: DONE
- **Last completed**: #4 — Run full validation
- **Current artifact**: `TODO.csv`
- **Key context**: Worktree started clean on `main...origin/main`. Need add card code currency snapshot and current-system-currency redemption conversion.
- **Known issues**: Frontend build may still log an existing unauthorized Axios message while exiting 0.
- **Next action**: Push `main` to origin.

## Milestone 1: Inspect currency and card-code integration points

- **Status**: DONE
- **Started**: 15:12
- **Completed**: 15:25
- **What was done**:
  - Located card code generation, storage, redemption, frontend table/export, system setting currency, and USD/CNY exchange-rate cache paths.
- **Key decisions**:
  - Add card code snapshot currency and use current system currency at redemption.
  - Extend wallet currency DB check from only CNY to USD/CNY because redemption can target either system currency.
- **Validation**: read-only `rg`/`sed` inspection
- **Next step**: Milestone 2/#3 — implement backend and frontend changes.

## Milestone 2/3: Implement Backend And Frontend

- **Status**: DONE
- **Started**: 15:25
- **Completed**: 15:51
- **What was done**:
  - Added card code `currency` snapshot across schema, entity, types, API response, and frontend type.
  - Injected current system currency into card code generation and redemption.
  - Added USD/CNY exchange conversion for card code amounts and existing wallet balances when target currency differs.
  - Updated card code table, export, and generation dialog amount label to show snapshot/system currency.
- **Validation**: covered by Milestone 4.
- **Next step**: Milestone 4 — full validation.

## Milestone 4: Run Full Validation

- **Status**: DONE
- **Started**: 15:51
- **Completed**: 15:54
- **Validation**: `cargo test -q -p card_code && cargo test -q -p storage card_code::redemption_currency && cargo check -q && pnpm lint:frontend && pnpm build:frontend && git diff --check` → exit 0
- **Known output**: `pnpm build:frontend` still prints the existing `Axios error: unauthorized` during static generation, but exits 0.
- **Next step**: Milestone 5 — commit amend and push.

## Milestone 5: Commit Amend And Push

- **Status**: DONE
- **Started**: 15:54
- **Completed**: 16:03
- **What was done**:
  - Created commit `feat: 添加工单/公单/兑换码系统`.
  - Rebasing found `origin/main` already had `56e6674 feat: 添加模型映射功能`; the card-code commit was replayed on top to avoid overwriting that remote work.
- **Validation**: full validation was rerun after rebase and passed.
- **Next step**: push `main`.

## Final Summary

- **Total milestones**: 5
- **Completed**: 5
- **Failed + recovered**: 0
- **External unblock events**: 1 remote commit preserved by rebase
- **Total retries**: 0
- **Files created**: 7
- **Files modified**: backend, frontend, storage, types, i18n seed, and task records
- **Key learnings**:
  - Card code currency has to be a persisted snapshot because system display currency can change before redemption.
