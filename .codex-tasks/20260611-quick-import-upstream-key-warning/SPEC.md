# Quick Import Upstream Key Warning

Clarify the quick-import sync status that currently appears as "upstream key unavailable" and ensure it does not disable local provider keys by default.

## Scope

- Work in the existing `codex/quick-import-provider` worktree.
- Confirm model candidate notifications are warning-only.
- Keep source fetch failures and true upstream token/model removal policies unchanged.
- Make the key-level fetch failure notification explain the failed operation and include the original error.

## Validation

- Targeted Rust tests for quick import sync outcome/policy/event behavior.
- Frontend lint or targeted TypeScript validation when feasible.
