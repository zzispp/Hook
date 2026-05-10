use axum::{
    Router,
    routing::{get, post},
};

use crate::api::{
    WalletApiState,
    handlers::{admin_adjust_wallet, admin_balance, admin_ledger, admin_recharge_wallet, admin_transactions, admin_wallets, balance, transactions},
};

pub fn create_router(state: WalletApiState) -> Router {
    Router::new()
        .route("/wallet/balance", get(balance))
        .route("/wallet/transactions", get(transactions))
        .route("/admin/wallets", get(admin_wallets))
        .route("/admin/wallets/users/{user_id}/balance", get(admin_balance))
        .route("/admin/wallets/ledger", get(admin_ledger))
        .route("/admin/wallets/{id}/transactions", get(admin_transactions))
        .route("/admin/wallets/{id}/adjust", post(admin_adjust_wallet))
        .route("/admin/wallets/{id}/recharge", post(admin_recharge_wallet))
        .with_state(state)
}
