use async_trait::async_trait;
use storage::{
    Database, StorageError,
    wallet::{WalletLedgerRecordInput, WalletStore, WalletTransactionRecordInput},
};
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    wallet::{Wallet, WalletTransaction},
};

use crate::application::{WalletError, WalletRepository, WalletResult};

#[derive(Clone)]
pub struct StorageWalletRepository {
    store: WalletStore,
}

impl StorageWalletRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: WalletStore::new(database),
        }
    }
}

#[async_trait]
impl WalletRepository for StorageWalletRepository {
    async fn find_by_user_id(&self, user_id: &str) -> WalletResult<Option<Wallet>> {
        self.store.find_by_user_id(user_id).await.map_err(storage_error)
    }

    async fn find_by_id(&self, wallet_id: &str) -> WalletResult<Option<Wallet>> {
        self.store.find_by_id(wallet_id).await.map_err(storage_error)
    }

    async fn ensure_user_wallet(&self, user_id: &str) -> WalletResult<Wallet> {
        self.store.ensure_user_wallet(user_id).await.map_err(storage_error)
    }

    async fn save_ledger_entry(&self, wallet: Wallet, transaction: WalletTransaction) -> WalletResult<WalletTransaction> {
        self.store
            .update_balances_with_transaction(ledger_input(wallet, transaction))
            .await
            .map_err(storage_error)
    }

    async fn page_transactions(&self, wallet_id: &str, page: PageRequest) -> WalletResult<Page<WalletTransaction>> {
        self.store.page_transactions(wallet_id, page_slice_request(page)).await.map_err(storage_error)
    }
}

fn ledger_input(wallet: Wallet, transaction: WalletTransaction) -> WalletLedgerRecordInput {
    WalletLedgerRecordInput {
        wallet,
        transaction: record_input(transaction),
    }
}

fn record_input(transaction: WalletTransaction) -> WalletTransactionRecordInput {
    WalletTransactionRecordInput {
        wallet_id: transaction.wallet_id,
        category: transaction.category,
        reason_code: transaction.reason_code,
        amount: transaction.amount,
        balance_before: transaction.balance_before,
        balance_after: transaction.balance_after,
        recharge_balance_before: transaction.recharge_balance_before,
        recharge_balance_after: transaction.recharge_balance_after,
        gift_balance_before: transaction.gift_balance_before,
        gift_balance_after: transaction.gift_balance_after,
        link_type: transaction.link_type,
        link_id: transaction.link_id,
        operator_id: transaction.operator_id,
        description: transaction.description,
    }
}

fn storage_error(error: StorageError) -> WalletError {
    match error {
        StorageError::NotFound => WalletError::NotFound,
        StorageError::Conflict(message) => WalletError::Conflict(message),
        StorageError::Database(message) => WalletError::Infrastructure(message),
    }
}

fn page_slice_request(page: PageRequest) -> PageSliceRequest {
    PageSliceRequest {
        offset: (page.page - 1) * page.page_size,
        limit: page.page_size,
        page: page.page,
        page_size: page.page_size,
    }
}
