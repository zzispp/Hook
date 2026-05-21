use rust_decimal::Decimal;
use sea_orm::FromQueryResult;
use time::format_description::well_known::Rfc3339;
use types::wallet::{AdminWalletLedgerEntryResponse, WalletLedgerEntry, WalletLedgerEntryKind, WalletLedgerEntryResponse, WalletTransaction};

#[derive(Debug, FromQueryResult)]
pub(super) struct LedgerEntryRow {
    entry_kind: String,
    id: String,
    wallet_id: String,
    category: String,
    reason_code: String,
    amount: Decimal,
    balance_before: Decimal,
    balance_after: Decimal,
    recharge_balance_before: Decimal,
    recharge_balance_after: Decimal,
    gift_balance_before: Decimal,
    gift_balance_after: Decimal,
    link_type: Option<String>,
    link_id: Option<String>,
    operator_id: Option<String>,
    description: Option<String>,
    created_at: time::OffsetDateTime,
    local_date: Option<String>,
    transaction_count: i64,
    first_created_at: time::OffsetDateTime,
    last_created_at: time::OffsetDateTime,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct AdminLedgerEntryRow {
    entry_kind: String,
    id: String,
    wallet_id: String,
    category: String,
    reason_code: String,
    amount: Decimal,
    balance_before: Decimal,
    balance_after: Decimal,
    recharge_balance_before: Decimal,
    recharge_balance_after: Decimal,
    gift_balance_before: Decimal,
    gift_balance_after: Decimal,
    link_type: Option<String>,
    link_id: Option<String>,
    operator_id: Option<String>,
    description: Option<String>,
    created_at: time::OffsetDateTime,
    local_date: Option<String>,
    transaction_count: i64,
    first_created_at: time::OffsetDateTime,
    last_created_at: time::OffsetDateTime,
    currency: String,
    owner_name: String,
    owner_email: String,
    owner_type: String,
    wallet_status: String,
}

impl From<LedgerEntryRow> for WalletLedgerEntry {
    fn from(row: LedgerEntryRow) -> Self {
        let first_created_at = format_timestamp(row.first_created_at);
        let last_created_at = format_timestamp(row.last_created_at);
        Self {
            entry_kind: ledger_entry_kind(&row.entry_kind),
            transaction: transaction_from_row(&row),
            local_date: row.local_date,
            transaction_count: row.transaction_count,
            first_created_at,
            last_created_at,
        }
    }
}

impl From<AdminLedgerEntryRow> for AdminWalletLedgerEntryResponse {
    fn from(row: AdminLedgerEntryRow) -> Self {
        let currency = row.currency.clone();
        let owner_name = row.owner_name.clone();
        let owner_email = row.owner_email.clone();
        let owner_type = row.owner_type.clone();
        let wallet_status = row.wallet_status.clone();
        Self {
            entry: WalletLedgerEntryResponse::from(WalletLedgerEntry::from(row)),
            currency,
            owner_name,
            owner_email,
            owner_type,
            wallet_status,
        }
    }
}

impl From<AdminLedgerEntryRow> for WalletLedgerEntry {
    fn from(row: AdminLedgerEntryRow) -> Self {
        LedgerEntryRow {
            entry_kind: row.entry_kind,
            id: row.id,
            wallet_id: row.wallet_id,
            category: row.category,
            reason_code: row.reason_code,
            amount: row.amount,
            balance_before: row.balance_before,
            balance_after: row.balance_after,
            recharge_balance_before: row.recharge_balance_before,
            recharge_balance_after: row.recharge_balance_after,
            gift_balance_before: row.gift_balance_before,
            gift_balance_after: row.gift_balance_after,
            link_type: row.link_type,
            link_id: row.link_id,
            operator_id: row.operator_id,
            description: row.description,
            created_at: row.created_at,
            local_date: row.local_date,
            transaction_count: row.transaction_count,
            first_created_at: row.first_created_at,
            last_created_at: row.last_created_at,
        }
        .into()
    }
}

fn transaction_from_row(row: &LedgerEntryRow) -> WalletTransaction {
    WalletTransaction {
        id: row.id.clone(),
        wallet_id: row.wallet_id.clone(),
        category: row.category.clone(),
        reason_code: row.reason_code.clone(),
        amount: row.amount,
        balance_before: row.balance_before,
        balance_after: row.balance_after,
        recharge_balance_before: row.recharge_balance_before,
        recharge_balance_after: row.recharge_balance_after,
        gift_balance_before: row.gift_balance_before,
        gift_balance_after: row.gift_balance_after,
        link_type: row.link_type.clone(),
        link_id: row.link_id.clone(),
        operator_id: row.operator_id.clone(),
        description: row.description.clone(),
        created_at: format_timestamp(row.created_at),
    }
}

fn ledger_entry_kind(value: &str) -> WalletLedgerEntryKind {
    match value {
        "daily_model_usage" => WalletLedgerEntryKind::DailyModelUsage,
        _ => WalletLedgerEntryKind::Transaction,
    }
}

fn format_timestamp(value: time::OffsetDateTime) -> String {
    value.format(&Rfc3339).expect("wallet ledger timestamp must format as RFC3339")
}
