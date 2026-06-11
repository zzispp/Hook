use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest},
    wallet::{
        AdminWalletConsumptionSummaryItem, AdminWalletConsumptionSummaryResponse, AdminWalletDailyUsageDetailsResponse,
        AdminWalletLedgerEntriesForWalletResponse, AdminWalletLedgerEntriesResponse, AdminWalletLedgerEntryResponse, AdminWalletLedgerFilters,
        AdminWalletLedgerResponse, AdminWalletLedgerTransactionResponse, AdminWalletListFilters, AdminWalletListResponse, AdminWalletResponse,
        AdminWalletTransactionsResponse, Wallet, WalletAdjustment, WalletBalanceResponse, WalletDailyUsageDetailRequest, WalletDailyUsageDetailsResponse,
        WalletLedgerEntriesResponse, WalletLedgerEntry, WalletLedgerEntryFilters, WalletRecharge, WalletTransaction, WalletTransactionsResponse,
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
    async fn page_ledger_entries(
        &self,
        wallet_id: &str,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<Page<WalletLedgerEntry>>;
    async fn page_daily_usage_transactions(
        &self,
        wallet_id: &str,
        page: PageRequest,
        request: WalletDailyUsageDetailRequest,
    ) -> WalletResult<Page<WalletTransaction>>;
    async fn find_admin_wallet_by_id(&self, wallet_id: &str) -> WalletResult<Option<AdminWalletResponse>>;
    async fn page_admin_wallets(&self, page: types::pagination::PageSliceRequest, filters: AdminWalletListFilters) -> WalletResult<Page<AdminWalletResponse>>;
    async fn page_admin_ledger(&self, page: PageRequest, filters: AdminWalletLedgerFilters) -> WalletResult<Page<AdminWalletLedgerTransactionResponse>>;
    async fn page_admin_ledger_entries(
        &self,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<Page<AdminWalletLedgerEntryResponse>>;
    async fn page_admin_consumption_summary(
        &self,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
    ) -> WalletResult<Page<AdminWalletConsumptionSummaryItem>>;
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
    async fn ledger_entries(
        &self,
        user_id: &str,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<WalletLedgerEntriesResponse>;
    async fn daily_usage_transactions(
        &self,
        user_id: &str,
        page: PageRequest,
        request: WalletDailyUsageDetailRequest,
    ) -> WalletResult<WalletDailyUsageDetailsResponse>;
    async fn admin_wallets(&self, page: PageRequest, filters: AdminWalletListFilters) -> WalletResult<AdminWalletListResponse>;
    async fn admin_ledger(&self, page: PageRequest, filters: AdminWalletLedgerFilters) -> WalletResult<AdminWalletLedgerResponse>;
    async fn admin_ledger_entries(
        &self,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<AdminWalletLedgerEntriesResponse>;
    async fn admin_consumption_summary(
        &self,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<AdminWalletConsumptionSummaryResponse>;
    async fn admin_transactions(&self, wallet_id: &str, page: PageRequest) -> WalletResult<AdminWalletTransactionsResponse>;
    async fn admin_ledger_entries_for_wallet(
        &self,
        wallet_id: &str,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<AdminWalletLedgerEntriesForWalletResponse>;
    async fn admin_daily_usage_transactions(
        &self,
        wallet_id: &str,
        page: PageRequest,
        request: WalletDailyUsageDetailRequest,
    ) -> WalletResult<AdminWalletDailyUsageDetailsResponse>;
    async fn adjust_wallet(&self, input: WalletAdjustment) -> WalletResult<WalletTransaction>;
    async fn recharge_wallet(&self, input: WalletRecharge) -> WalletResult<WalletTransaction>;
}
