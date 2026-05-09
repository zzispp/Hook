use axum::{Router, routing::get};

use crate::api::{
    WalletApiState,
    handlers::{balance, transactions},
};

pub fn create_router(state: WalletApiState) -> Router {
    Router::new()
        .route("/wallet/balance", get(balance))
        .route("/wallet/transactions", get(transactions))
        .with_state(state)
}
