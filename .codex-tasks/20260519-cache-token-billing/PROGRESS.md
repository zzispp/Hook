# Progress Log

## Session Start

- **Date**: 2026-05-19 11:18 CST
- **Task name**: `20260519-cache-token-billing`
- **Task dir**: `.codex-tasks/20260519-cache-token-billing/`
- **Spec**: See `SPEC.md`
- **Plan**: See `TODO.csv`
- **Environment**: Rust workspace / cargo test

## Context Recovery Block

- **Current milestone**: #7 — Audit other endpoint families
- **Current status**: DONE
- **Last completed**: #7 — Audit other endpoint families
- **Current artifact**: `TODO.csv`
- **Key context**: Hook default billing and request-record persistence now use non-cache input tokens for OpenAI/Gemini-style usage. Request records preserve cache creation/read token fields separately. Claude/Anthropic input tokens are not reduced because upstream `input_tokens` already exclude cache creation/read. Rerank has no cache dimensions. GeminiVideo currently emits no `TokenUsage`.
- **Known issues**: `cargo test -p backend usage_fields` is blocked by unrelated current `crates/user` compile errors in registration email code (`K` generic missing, trait method missing, record initializer fields missing). `cargo fmt --check -p backend`, `cargo fmt --check -p provider`, provider tests, frontend lint, and frontend typecheck pass.
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

## Milestone 5: Align request-record usage writes

- **Status**: DONE
- **Started**: 13:30
- **Completed**: 13:40
- **What was done**:
  - Updated audit request/candidate record mapping so persisted `prompt_tokens` is display/billable input for OpenAI/Gemini-style usage.
  - Preserved `cache_creation_input_tokens` and `cache_read_input_tokens` as separate fields.
  - Recomputed persisted `total_tokens` by subtracting cache tokens from raw total when raw total exists, preserving input-only endpoints such as embeddings.
- **Endpoint audit**:
  - OpenAI chat, responses/CLI, completion, embedding, image, audio, moderation, and realtime include cache tokens in prompt/input semantics when cache fields are present.
  - Gemini chat and embedding include cached content in prompt semantics when `cachedContentTokenCount` is present.
  - Claude/Anthropic is unchanged because `input_tokens` is already non-cache input while cache creation/read are separate fields.
  - Rerank has no cache dimensions. GeminiVideo currently returns no usage.
- **Files changed**:
  - `apps/hook_backend/src/llm_proxy/audit/records/usage_fields.rs`
- **Validation**:
  - `cargo fmt --check -p backend` → passed
  - `perl -e 'alarm 60; exec @ARGV' cargo test -p backend usage_fields` → blocked by unrelated `crates/user` compile errors.

## Milestone 6: Add drawer token quantities

- **Status**: DONE
- **Started**: 13:40
- **Completed**: 13:42
- **What was done**:
  - Request record table and drawer header display persisted `prompt_tokens / completion_tokens` directly.
  - Billing detail drawer now shows input/output/cache-creation/cache-read token quantities above cost rows.
  - Added backend i18n seed labels for the new token quantity fields.
- **Files changed**:
  - `apps/hook_frontend/src/sections/admin/request-record-billing-details.tsx`
  - `apps/hook_frontend/src/sections/admin/request-record-detail-drawer.tsx`
  - `apps/hook_backend/src/migration/defaults/i18n/admin.cn.json`
  - `apps/hook_backend/src/migration/defaults/i18n/admin.en.json`
- **Validation**:
  - `pnpm --filter hook_frontend lint` → passed
  - `pnpm --filter hook_frontend exec tsc --noEmit` → passed

## Milestone 7: Validate endpoint coverage

- **Status**: DONE
- **Started**: 13:42
- **Completed**: 13:45
- **What was done**:
  - Verified non-stream, stream, estimated stream, and websocket realtime success records converge through `AttemptRecordInput.usage` and `usage_fields`.
  - Verified provider billing normalization is already in current baseline and provider tests cover OpenAI, Gemini, Claude, and explicit billing rule behavior.
- **Validation**:
  - `cargo fmt --check -p provider` → passed
  - `perl -e 'alarm 60; exec @ARGV' cargo test -p provider` → passed, 40 tests
