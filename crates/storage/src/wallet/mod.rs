mod admin_repository;
mod consume;
pub mod record;
mod repository;
mod types;

pub use repository::WalletStore;
pub use types::{
    AdminWalletLedgerRecord, AdminWalletRecord, WALLET_CONSUME_INSUFFICIENT_BALANCE, WalletConsumeRecordInput, WalletLedgerRecordInput,
    WalletTransactionRecordInput,
};

pub(crate) use record::{WalletRecord, WalletTransactionRecord, wallet_transactions as wallet_transaction_records, wallets as wallet_records};
