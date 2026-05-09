# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Match aether's admin wallet management shape for the current Hook scope.
- Add two tabs to Hook admin wallet management: wallet list and global ledger.
- Global ledger must list all wallet transactions, not only transactions for one wallet.
- Global ledger rows must show owner, category/reason, amount, balance change, description, and open the existing detail dialog.
- Add backend API and baseline permission for global admin wallet ledger.

## Non-Goals

- Do not implement aether tabs outside the requested scope, such as redeem codes, packages, orders, refunds, or callbacks.
- Do not include unrelated auth/session worktree changes.

## Constraints

- Keep files <= 300 lines and functions <= 50 lines.
- Use existing Hook MUI/Next patterns and existing wallet display helpers.
- Keep local failures visible; no mock or fallback success paths.

## Done-When

- `/dashboard/admin/wallets` has wallet-list and global-ledger tabs.
- `GET /api/admin/wallets/ledger` returns paginated global ledger with owner metadata.
- Baseline API permissions include the ledger endpoint.
- Frontend lint/build and backend checks/tests pass where feasible.
