use rust_decimal::Decimal;
use serde::Serialize;

use super::{AdminWalletResponse, WalletSummaryResponse, WalletTransaction, WalletTransactionResponse};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalletLedgerEntry {
    pub entry_kind: WalletLedgerEntryKind,
    pub transaction: WalletTransaction,
    pub local_date: Option<String>,
    pub transaction_count: i64,
    pub first_created_at: String,
    pub last_created_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletLedgerEntryKind {
    Transaction,
    DailyModelUsage,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WalletLedgerEntryResponse {
    #[serde(flatten)]
    pub transaction: WalletTransactionResponse,
    pub entry_kind: WalletLedgerEntryKind,
    pub local_date: Option<String>,
    pub transaction_count: i64,
    pub first_created_at: String,
    pub last_created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminWalletLedgerEntryResponse {
    #[serde(flatten)]
    pub entry: WalletLedgerEntryResponse,
    pub currency: String,
    pub owner_name: String,
    pub owner_email: String,
    pub owner_type: String,
    pub wallet_status: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WalletLedgerEntryFilters {
    pub search: Option<String>,
    pub category: Option<String>,
    pub reason_code: Option<String>,
    pub direction: Option<String>,
    pub balance_type: Option<String>,
    pub link_type: Option<String>,
    pub owner_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalletDailyUsageDetailRequest {
    pub local_date: String,
    pub tz_offset_minutes: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WalletLedgerEntriesResponse {
    pub wallet: WalletSummaryResponse,
    pub items: Vec<WalletLedgerEntryResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WalletDailyUsageDetailsResponse {
    pub items: Vec<WalletTransactionResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminWalletLedgerEntriesResponse {
    pub items: Vec<AdminWalletLedgerEntryResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminWalletLedgerEntriesForWalletResponse {
    pub wallet: AdminWalletResponse,
    pub items: Vec<WalletLedgerEntryResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminWalletDailyUsageDetailsResponse {
    pub wallet: AdminWalletResponse,
    pub items: Vec<WalletTransactionResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl From<WalletLedgerEntry> for WalletLedgerEntryResponse {
    fn from(value: WalletLedgerEntry) -> Self {
        Self {
            entry_kind: value.entry_kind,
            transaction: WalletTransactionResponse::from(value.transaction),
            local_date: value.local_date,
            transaction_count: value.transaction_count,
            first_created_at: value.first_created_at,
            last_created_at: value.last_created_at,
        }
    }
}

pub fn ledger_amount_total(items: &[WalletLedgerEntry]) -> Decimal {
    items.iter().map(|item| item.transaction.amount).sum()
}
