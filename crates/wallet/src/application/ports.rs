use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest},
    wallet::{
        AdminWalletLedgerFilters, AdminWalletLedgerResponse, AdminWalletLedgerTransactionResponse, AdminWalletListFilters, AdminWalletListResponse,
        AdminWalletResponse, AdminWalletTransactionsResponse, Wallet, WalletAdjustment, WalletBalanceResponse, WalletRecharge, WalletTransaction,
        WalletTransactionsResponse,
    },
};

use super::WalletResult;

/// Persists wallet current state and append-only ledger records.
///
/// Implementations must expose storage failures and must not fake wallet creation
/// or ledger writes. Balance mutations are coordinated by the application service.
#[async_trait]
pub trait WalletRepository: Send + Sync + 'static {
    async fn find_by_user_id(&self, user_id: &str) -> WalletResult<Option<Wallet>>;
    async fn find_by_id(&self, wallet_id: &str) -> WalletResult<Option<Wallet>>;
    async fn ensure_user_wallet(&self, user_id: &str) -> WalletResult<Wallet>;
    async fn save_ledger_entry(&self, wallet: Wallet, transaction: WalletTransaction) -> WalletResult<WalletTransaction>;
    async fn page_transactions(&self, wallet_id: &str, page: PageRequest) -> WalletResult<Page<WalletTransaction>>;
    async fn find_admin_wallet_by_id(&self, wallet_id: &str) -> WalletResult<Option<AdminWalletResponse>>;
    async fn page_admin_wallets(&self, page: types::pagination::PageSliceRequest, filters: AdminWalletListFilters) -> WalletResult<Page<AdminWalletResponse>>;
    async fn page_admin_ledger(&self, page: PageRequest, filters: AdminWalletLedgerFilters) -> WalletResult<Page<AdminWalletLedgerTransactionResponse>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemWalletRecord {
    pub wallet: Wallet,
    pub owner_name: String,
    pub owner_email: String,
}

pub trait SystemWalletProvider: Clone + Send + Sync + 'static {
    fn system_wallet(&self) -> Option<SystemWalletRecord>;
}

#[derive(Clone, Copy, Debug)]
pub struct NoSystemWalletProvider;

impl SystemWalletProvider for NoSystemWalletProvider {
    fn system_wallet(&self) -> Option<SystemWalletRecord> {
        None
    }
}

#[async_trait]
pub trait WalletUseCase: Send + Sync + 'static {
    async fn balance(&self, user_id: &str) -> WalletResult<WalletBalanceResponse>;
    async fn admin_balance(&self, user_id: &str) -> WalletResult<WalletBalanceResponse>;
    async fn transactions(&self, user_id: &str, page: PageRequest) -> WalletResult<WalletTransactionsResponse>;
    async fn admin_wallets(&self, page: PageRequest, filters: AdminWalletListFilters) -> WalletResult<AdminWalletListResponse>;
    async fn admin_ledger(&self, page: PageRequest, filters: AdminWalletLedgerFilters) -> WalletResult<AdminWalletLedgerResponse>;
    async fn admin_transactions(&self, wallet_id: &str, page: PageRequest) -> WalletResult<AdminWalletTransactionsResponse>;
    async fn adjust_wallet(&self, input: WalletAdjustment) -> WalletResult<WalletTransaction>;
    async fn recharge_wallet(&self, input: WalletRecharge) -> WalletResult<WalletTransaction>;
}
