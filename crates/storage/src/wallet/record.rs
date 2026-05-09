#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::{wallet_transactions, wallets};

pub type WalletRecord = wallets::Model;
pub type WalletTransactionRecord = wallet_transactions::Model;
