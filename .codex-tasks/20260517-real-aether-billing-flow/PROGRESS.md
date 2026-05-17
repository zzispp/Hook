# Progress Log

---

## Session Start

- **Date**: 2026-05-17
- **Task name**: `20260517-real-aether-billing-flow`
- **Task dir**: `.codex-tasks/20260517-real-aether-billing-flow/`
- **Spec**: See `SPEC.md`
- **Plan**: See `TODO.csv`
- **Environment**: local Postgres Docker container / Redis / backend / real upstream providers

---

## Context Recovery Block

- **Current milestone**: #1 - Create real billing flow task artifacts
- **Current status**: IN_PROGRESS
- **Last completed**: none
- **Current artifact**: `TODO.csv`
- **Key context**: Build and run a real script that validates Aether-style billing snapshots, grouped cost, wallet settlement, token usage, model usage, and request records.
- **Known issues**: local host has no `psql`; use Docker Postgres `hook-postgres`.
- **Next action**: Implement the Node E2E script with environment-only provider secrets.

---

## Milestone 1: Create real billing flow task artifacts

- **Status**: DONE
- **Completed**: 2026-05-17
- **What was done**:
  - Created `SPEC.md`, `TODO.csv`, `PROGRESS.md`, and `raw/`.
- **Validation**: task artifact `test -f ...` checks pass.

---

## Milestone 2: Implement real E2E billing script

- **Status**: DONE
- **Completed**: 2026-05-17
- **What was done**:
  - Added `real_aether_billing_flow.mjs` and small helper modules under `lib/`.
  - Script reads upstream keys only from environment variables.
  - Script probes real `/v1/models`, seeds local DB, sends proxy requests, validates billing snapshots, wallet settlement, token quota, model usage, and request records.
  - Script writes non-secret evidence to `raw/results.json`.
- **Validation**:
  - `node --check .codex-tasks/20260517-real-aether-billing-flow/real_aether_billing_flow.mjs` -> exit 0.
  - `node --check` for every `lib/*.mjs` -> exit 0.
  - Secret grep for the provided key prefixes -> no matches.

---

## Milestone 3: Run local DB and real upstream validation

- **Status**: DONE
- **Started**: 2026-05-17
- **Completed**: 2026-05-17
- **What was done**:
  - Ran `cargo run -p backend -- migration up` with a 60 second Perl alarm to upgrade the local DB schema.
  - Confirmed `billing_rules`, `dimension_collectors`, and `billing_snapshot` columns exist.
  - Ran the real E2E script with temporary exported provider keys.
  - Probed real upstream model lists and selected:
    - 86gamestore: `gpt-5.4-mini`
    - Ekan8 mapped model: `R-claude-opus-4-7`
  - Executed two real Hook proxy requests and verified request/candidate snapshots, group multiplier, wallet settlement, token usage, and model usage.
- **Debug notes**:
  - First run exposed that token/model usage is asynchronously flushed; the script now waits for flush completion before asserting aggregate DB columns.
  - A failed run left an empty Redis `processing:model:batch_id` without records; the script now clears empty processing batch ids during its own setup.
- **Validation**: `perl -e 'alarm shift; exec @ARGV' 180 node .codex-tasks/20260517-real-aether-billing-flow/real_aether_billing_flow.mjs` -> exit 0.

---

## Milestone 4: Record results and final verification

- **Status**: DONE
- **Started**: 2026-05-17
- **Completed**: 2026-05-17
- **What was done**:
  - Wrote non-secret execution evidence to `raw/results.json`.
  - Verified the result file exists and parses.
  - Verified secret grep has no matches for the supplied provider key prefixes.
  - Verified seeded test token, providers, models, and user were deactivated after the run.
- **Validation**:
  - `test -f .codex-tasks/20260517-real-aether-billing-flow/raw/results.json` -> exit 0.
  - Parsed result summary: 2 scenarios, total wallet/token usage `0.00124250`, token request count `2`.
  - Secret grep for supplied key prefixes -> no matches.
- **Final status**: DONE.
