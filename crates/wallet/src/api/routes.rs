use axum::{
    Router,
    routing::{get, post},
};

use crate::api::{
    WalletApiState,
    handlers::{
        admin_adjust_wallet, admin_balance, admin_consumption_summary, admin_daily_usage_transactions, admin_ledger, admin_ledger_entries,
        admin_ledger_entries_for_wallet, admin_recharge_wallet, admin_transactions, admin_wallets, balance, daily_usage_transactions, ledger_entries,
        transactions,
    },
};

pub fn create_router(state: WalletApiState) -> Router {
    Router::new()
        .route("/wallet/balance", get(balance))
        .route("/wallet/transactions", get(transactions))
        .route("/wallet/ledger-entries", get(ledger_entries))
        .route("/wallet/ledger-entries/daily-model-usage", get(daily_usage_transactions))
        .route("/admin/wallets", get(admin_wallets))
        .route("/admin/wallets/users/{user_id}/balance", get(admin_balance))
        .route("/admin/wallets/ledger", get(admin_ledger))
        .route("/admin/wallets/ledger-entries", get(admin_ledger_entries))
        .route("/admin/wallets/ledger-consumption-summary", get(admin_consumption_summary))
        .route("/admin/wallets/{id}/transactions", get(admin_transactions))
        .route("/admin/wallets/{id}/ledger-entries", get(admin_ledger_entries_for_wallet))
        .route("/admin/wallets/{id}/ledger-entries/daily-model-usage", get(admin_daily_usage_transactions))
        .route("/admin/wallets/{id}/adjust", post(admin_adjust_wallet))
        .route("/admin/wallets/{id}/recharge", post(admin_recharge_wallet))
        .with_state(state)
}
