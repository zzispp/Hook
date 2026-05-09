# Hook Wallet

## Goal

Add a CNY wallet system aligned with aether's wallet center fundamentals: every normal user can have a wallet, wallet balance is current state, wallet transactions are the auditable ledger, and the user role can see the wallet center without receiving admin wallet management access.

## Scope

- Add wallet types, storage entities, repository boundary, service, API routes, and backend wiring.
- Add SeaORM migration for `wallets` and `wallet_transactions`, including backfill for existing users.
- Add default API/menu permissions for the user wallet center.
- Add a frontend wallet center route with balance and transaction list.

## Out Of Scope

- Payment gateway orders.
- Refund workflow.
- Redeem codes.
- Daily usage ledger aggregation.
- Admin wallet management UI.
