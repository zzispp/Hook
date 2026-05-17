# Progress Log

---

## Session Start

- **Date**: 2026-05-17
- **Task name**: `aether-chain-alignment`
- **Task dir**: `.codex-tasks/aether-chain-alignment/`
- **Spec**: See `EPIC.md`
- **Plan**: See `SUBTASKS.csv`
- **Environment**: Rust workspace / SeaORM / Axum / just test wrapper

---

## Context Recovery Block

- **Current milestone**: #2 - Add billing schemas and storage fields
- **Current status**: IN_PROGRESS
- **Last completed**: #1 - Create Taskmaster epic tracking
- **Current artifact**: `SUBTASKS.csv`
- **Key context**: Implement Aether-style HTTP chain and billing alignment in Hook, preserving Hook cooldown/cache/group behavior.
- **Known issues**: none
- **Next action**: Add billing rule and dimension collector schema plus request billing snapshot fields.

---

## Milestone 1: Create Taskmaster epic tracking

- **Status**: DONE
- **Started**: session start
- **Completed**: 2026-05-17
- **What was done**:
  - Created `EPIC.md`, `SUBTASKS.csv`, and `PROGRESS.md`.
- **Validation**: `test -f ...` -> exit 0
- **Files changed**:
  - `.codex-tasks/aether-chain-alignment/EPIC.md`
  - `.codex-tasks/aether-chain-alignment/SUBTASKS.csv`
  - `.codex-tasks/aether-chain-alignment/PROGRESS.md`
- **Next step**: Milestone 2 - Add billing schemas and storage fields

---

## Milestone 2: Add billing schemas and storage fields

- **Status**: DONE
- **Completed**: 2026-05-17
- **What was done**:
  - Added `billing_rules` and `dimension_collectors` baseline schema, idens, indices, and development reset ordering.
  - Added SeaORM entities and provider storage methods for enabled rules and collectors.
  - Added `billing_snapshot` JSON fields to request records and request candidates.
  - Added provider model binding id into candidate trace so model-scoped billing rules can be resolved.
- **Validation**: `cargo check -p provider -p storage -p backend` -> exit 0. `timeout` is not installed on this host.
- **Next step**: Milestone 3 - Implement FormulaEngine collectors and BillingService

---

## Milestone 3: Implement FormulaEngine collectors and BillingService

- **Status**: DONE
- **Completed**: 2026-05-17
- **What was done**:
  - Added the Aether-style billing domain under `crates/provider/src/application/billing/`.
  - Implemented `FormulaEngine`, safe expression evaluation, collector runtime, rule service, and `BillingService::calculate_from_response`.
  - Added `BillingSnapshot` with base cost, group multiplier, resolved dimensions, variables, missing required dimensions, and cost breakdown.
  - Supported dimension, constant, matrix, tiered, and computed mappings with explicit incomplete/error behavior.
- **Validation**: `perl -e 'alarm shift; exec @ARGV' 60 cargo test --workspace` -> exit 0. `just`, `timeout`, and `gtimeout` are not installed on this host.

---

## Milestone 4: Wire billing into audit wallet and usage

- **Status**: DONE
- **Completed**: 2026-05-17
- **What was done**:
  - Wired billing runtime into LLM proxy audit records.
  - Persisted `billing_snapshot` to request and candidate records.
  - Derived request cost columns from `BillingSnapshot`.
  - Settled wallet, token usage, and model usage from new `BillingService` total cost only.
  - Made successful responses with incomplete billing explicit instead of silently charging zero.
- **Validation**: `perl -e 'alarm shift; exec @ARGV' 60 cargo test --workspace` -> exit 0.

---

## Milestone 5: Align failure classification with cooldown

- **Status**: DONE
- **Completed**: 2026-05-17
- **What was done**:
  - Added Aether-like failure classification for upstream HTTP responses.
  - Routed 401/403 to next candidate without provider cooldown writes.
  - Routed 408/429/5xx through retry/candidate switch behavior with existing cooldown recording.
  - Returned client request errors directly without meaningless retry or cooldown.
- **Validation**: `perl -e 'alarm shift; exec @ARGV' 60 cargo test --workspace` -> exit 0.

---

## Milestone 6: Clean old billing path and validate full chain

- **Status**: DONE
- **Completed**: 2026-05-17
- **What was done**:
  - Removed the old `calculate_request_billing` business path from active code.
  - Verified old billing identifiers only remain in historical task raw notes.
  - Ran formatting and workspace checks after integration.
- **Validation**:
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo check --workspace` -> exit 0.
  - `perl -e 'alarm shift; exec @ARGV' 60 cargo test --workspace` -> exit 0.
- **Final status**: Epic implementation complete.
