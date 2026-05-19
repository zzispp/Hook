# Progress Log

## Session Start

- **Date**: 2026-05-19 11:18 CST
- **Task name**: `20260519-cache-token-billing`
- **Task dir**: `.codex-tasks/20260519-cache-token-billing/`
- **Spec**: See `SPEC.md`
- **Plan**: See `TODO.csv`
- **Environment**: Rust workspace / cargo test

## Context Recovery Block

- **Current milestone**: #4 — Validate affected Rust checks
- **Current status**: DONE
- **Last completed**: #4 — Validate affected Rust checks
- **Current artifact**: `TODO.csv`
- **Key context**: Hook default billing now normalizes OpenAI/Gemini billable input tokens by subtracting separately priced cache tokens while preserving `total_input_context` from raw context semantics. Claude and explicit custom billing rules keep raw `input_tokens`.
- **Known issues**: `cargo fmt --check --all` still reports pre-existing unrelated formatting diffs under `crates/user`; provider package formatting passes.
- **Next action**: Final response to user.

## Milestone 1: Read constraints and establish task context

- **Status**: DONE
- **Started**: 11:18
- **Completed**: 11:18
- **What was done**:
  - Created task artifacts for cross-repository billing investigation.
- **Validation**: `test -f .codex-tasks/20260519-cache-token-billing/SPEC.md && test -f .codex-tasks/20260519-cache-token-billing/TODO.csv` → pending final run
- **Files changed**:
  - `.codex-tasks/20260519-cache-token-billing/*` — task tracking artifacts
- **Next step**: Milestone 2 — Compare aether and new-api cache-token billing semantics

## Milestone 2: Compare aether and new-api cache-token billing semantics

- **Status**: DONE
- **Started**: 11:18
- **Completed**: 11:21
- **What was done**:
  - Read `new-api/pkg/billingexpr/expr.md`, `service/tiered_settle.go`, and token normalization tests.
  - Read `aether/src/services/billing/token_normalization.py`, usage billing integration, and default billing rules.
- **Key decisions**:
  - Decision: Align Hook default billing to normalize计价输入 token, not request-record display token.
  - Reasoning: Both reference systems separate raw context length from billable non-cache input; cache read is billed through its own dimension.
- **Validation**: read-only evidence collected from local repositories
- **Files changed**:
  - none outside task tracking
- **Next step**: Milestone 3 — Implement Hook billing alignment

## Milestone 3: Implement Hook billing alignment

- **Status**: DONE
- **Started**: 11:21
- **Completed**: 11:37
- **What was done**:
  - Updated provider billing dimension normalization so the default rule uses billable non-cache input tokens for OpenAI/Gemini API formats.
  - Preserved `raw_input_tokens` and computed `total_input_context` before billable-input reduction.
  - Kept Claude semantics unchanged because Claude input tokens are already non-cache text input.
  - Kept explicit model/global billing rules on raw `input_tokens` to avoid changing administrator-defined formulas.
- **Validation**:
  - `perl -e 'alarm 60; exec @ARGV' cargo test -p provider openai_cache_read_tokens_are_removed_from_billable_input` failed before the fix with `input_tokens = 146000`, then passed after the fix.
- **Files changed**:
  - `crates/provider/src/application/billing/service.rs`
  - `crates/provider/src/application/billing/service/tests.rs`
- **Next step**: Milestone 4 — Validate affected Rust checks

## Milestone 4: Validate affected Rust checks

- **Status**: DONE
- **Started**: 11:37
- **Completed**: 11:37
- **Validation**:
  - `cargo fmt --check -p provider` → passed
  - `perl -e 'alarm 60; exec @ARGV' cargo test -p provider` → passed, 38 tests
- **Notes**:
  - `timeout` and `gtimeout` are not available in this macOS shell, so validation used Perl `alarm 60`.
  - `cargo fmt --check --all` reports unrelated existing formatting differences in `crates/user/src/application/service.rs`; those files were not changed for this task.
