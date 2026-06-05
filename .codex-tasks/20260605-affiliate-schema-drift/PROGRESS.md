# Progress

## 2026-06-05

- User startup failed because system_settings.affiliate_min_commission_amount was missing in the existing local Postgres database.
- information_schema confirmed system_settings had affiliate_enabled and affiliate_commission_percent only.
- affiliate_commissions also used the older shape without status/failure_reason and with wallet_transaction_id NOT NULL.
- Patched the local Postgres schema in place: added system_settings.affiliate_min_commission_amount, made affiliate_commissions.wallet_transaction_id nullable, added status/failure_reason, and added explicit outcome constraints.
- Synced development migration table metadata with affiliate_relation_changes and affiliate_commissions.
- Validation passed: cargo check -p backend; cargo fmt --all --check; timeout startup reached backend listening before timeout stopped it.
