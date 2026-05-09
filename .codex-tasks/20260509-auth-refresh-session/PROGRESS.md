# Progress Log

> Auto-maintained by Taskmaster. Each entry records what happened, why, and what's next.
> This file serves as both decision audit trail and context-recovery anchor.

---

## Session Start

- **Date**: 2026-05-09 17:28
- **Task name**: `auth-refresh-session`
- **Task dir**: `.codex-tasks/20260509-auth-refresh-session/`
- **Spec**: See SPEC.md
- **Plan**: See TODO.csv (4 milestones)
- **Environment**: Rust 2024, TypeScript/Next.js, cargo test, pnpm lint

---

## Context Recovery Block

> If you are resuming this task after compaction, session restart, or context loss,
> read this section FIRST to restore working state.

- **Current milestone**: #6 — Run final frontend validation
- **Current status**: DONE
- **Last completed**: #6 — Run final frontend validation
- **Current artifact**: `.codex-tasks/20260509-auth-refresh-session/TODO.csv`
- **Key context**: Backend user API maps `InvalidCredentials` and `Unauthorized` to HTTP 401. Frontend refresh and `/me` 401 clear local JWT session. Axios now pre-refreshes near expiry and retries stale-token 401 once.
- **Known issues**: Existing unrelated wallet changes are present in the worktree and must not be touched.
- **Next action**: None for this task.

> Update this block EVERY TIME a milestone changes status.

---

<!-- Append entries below as each milestone completes -->

## Milestone 1: Confirm current contract and failing expectations

- **Status**: DONE
- **Started**: 17:28
- **Completed**: 17:36
- **What was done**:
  - Added failing expectations for HTTP 401 on auth failures and distinct refresh/sign-in messages.
- **Key decisions**:
  - Decision: Treat refresh token rejection as unauthorized, not invalid credentials.
  - Reasoning: Refresh does not involve username or password.
  - Alternatives considered: Frontend-only body parsing; rejected because it leaves backend contract misleading.
- **Problems encountered**:
  - Problem: `timeout` command is unavailable on macOS.
  - Resolution: Used a Perl process-group timeout wrapper with a 60-second limit.
  - Retry count: 1
- **Validation**: Perl timeout wrapper `cargo test -p user api::error::tests::api_error_uses_unauthorized_status` → exit 101 before implementation, expected failure.
- **Files changed**:
  - `crates/user/src/api/error.rs` — changed test expectation.
  - `crates/user/src/api/routes/tests.rs` — added/updated route expectations.
- **Next step**: Milestone 2 — Implement backend auth error semantics

---

## Milestone 2: Implement backend auth error semantics

- **Status**: DONE
- **Started**: 17:36
- **Completed**: 17:39
- **What was done**:
  - Added `InvalidCredentials` for sign-in failures.
  - Mapped user API invalid credentials and unauthorized errors to HTTP 401.
- **Key decisions**:
  - Decision: Preserve the API envelope shape while fixing HTTP status.
  - Reasoning: The frontend and existing actions already consume `{ success, message, data }`.
- **Problems encountered**:
  - Problem: Old application-layer invalid-password test still expected `Unauthorized`.
  - Resolution: Update it in the next code edit to match the new typed error.
  - Retry count: 0
- **Validation**: API target tests for auth status/message → exit 0.
- **Files changed**:
  - `crates/user/src/application/error.rs` — added `InvalidCredentials`.
  - `crates/user/src/application/service.rs` — sign-in failures now use `InvalidCredentials`.
  - `crates/user/src/api/error.rs` — added status mapping.
- **Next step**: Milestone 3 — Implement frontend refresh session expiry handling

---

<!-- Final summary goes here when all milestones are DONE -->

## Milestone 3: Implement frontend refresh session expiry handling

- **Status**: DONE
- **Started**: 17:39
- **Completed**: 17:48
- **What was done**:
  - Preserved HTTP status on axios errors through `ApiRequestError`.
  - Made refresh 401 and `/me` 401 return null session so `checkUserSession` clears local tokens.
- **Key decisions**:
  - Decision: Only 401 becomes session expiry; other errors still throw.
  - Reasoning: Network, backend, and response-shape failures should remain visible.
- **Problems encountered**:
  - Problem: None in the touched frontend files.
  - Resolution: N/A.
  - Retry count: 0
- **Validation**: `pnpm --dir apps/hook_frontend exec eslint src/auth/context/jwt/auth-provider.tsx src/lib/axios.ts` → exit 0.
- **Files changed**:
  - `apps/hook_frontend/src/lib/axios.ts` — preserves HTTP status in typed errors.
  - `apps/hook_frontend/src/auth/context/jwt/auth-provider.tsx` — handles auth 401 as session expiry.
- **Next step**: Milestone 4 — Run final validation

---

## Milestone 4: Run final validation

- **Status**: DONE
- **Started**: 17:48
- **Completed**: 17:55
- **What was done**:
  - Ran user crate tests, frontend typecheck, and targeted frontend lint.
  - Checked broader lint/clippy and recorded unrelated blockers.
- **Key decisions**:
  - Decision: Do not modify unrelated wallet/storage files as part of this auth task.
  - Reasoning: The worktree already contains separate wallet changes, and changing them would expand scope.
- **Problems encountered**:
  - Problem: `pnpm lint:frontend` fails on `apps/hook_frontend/src/actions/wallet.ts` import sorting.
  - Resolution: Recorded as unrelated blocker.
  - Retry count: 0
  - Problem: `cargo clippy -p user --all-targets -- -D warnings` fails in storage crate on existing `needless_update` warnings.
  - Resolution: Recorded as unrelated blocker.
  - Retry count: 0
- **Validation**: `perl timeout wrapper 60 cargo test -p user` → exit 0, 33 passed.
- **Validation**: `pnpm --dir apps/hook_frontend exec eslint src/auth/context/jwt/auth-provider.tsx src/lib/axios.ts` → exit 0.
- **Validation**: `pnpm --dir apps/hook_frontend exec tsc --noEmit --pretty false` → exit 0.
- **Files changed**:
  - `.codex-tasks/20260509-auth-refresh-session/PROGRESS.md` — task record.
  - `.codex-tasks/20260509-auth-refresh-session/TODO.csv` — task status.
- **Next step**: none.

## Final Summary

- **Total milestones**: 6
- **Completed**: 6
- **Failed + recovered**: 0
- **External unblock events**: 2
- **Total retries**: 1
- **Files created**: 3
- **Files modified**: 8
- **Key learnings**:
  - Auth token failures need HTTP status semantics; body-only `success:false` is not enough for session handling.

## 2026-05-10 Runtime Refresh Follow-Up

- Added a JWT axios interceptor module that:
  - sends only the access token on ordinary API requests,
  - refreshes with the refresh token only against `/api/auth/refresh`,
  - refreshes before requests when the access token is already expired or within the configured refresh window,
  - retries a 401 response only once and only when the failed request used a stale access token.
- Kept retry behavior conservative for non-idempotent calls: a replay only happens for a stale-token authentication failure, not arbitrary 401/permission errors.
- Validation passed:
  - `pnpm --filter hook_frontend lint`
  - `pnpm --dir apps/hook_frontend exec tsc --noEmit --pretty false`
  - `pnpm --filter hook_frontend build`
  - `cargo check -q`
