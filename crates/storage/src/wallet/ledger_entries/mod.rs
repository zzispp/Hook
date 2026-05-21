mod query;
mod rows;

use sea_orm::{DbBackend, FromQueryResult, Statement};
use types::{
    pagination::{Page, PageSliceRequest},
    wallet::{
        AdminWalletLedgerEntryResponse, WalletDailyUsageDetailRequest, WalletLedgerEntry, WalletLedgerEntryFilters, WalletTransaction,
    },
};

use crate::StorageResult;

use super::{WalletStore, wallet_transaction_records};
use query::{daily_usage_count, daily_usage_filters, ledger_entry_count, ledger_entry_statement, pagination_value};
use rows::{AdminLedgerEntryRow, LedgerEntryRow};

impl WalletStore {
    pub async fn page_ledger_entries(
        &self,
        wallet_id: &str,
        request: PageSliceRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> StorageResult<Page<WalletLedgerEntry>> {
        let query = query::wallet_query(wallet_id, filters, tz_offset_minutes);
        let total = ledger_entry_count(self, query.filtered_sql(), query.values()).await?;
        let rows = LedgerEntryRow::find_by_statement(ledger_entry_statement(query, request))
            .all(self.database.connection())
            .await?;
        Ok(Page {
            items: rows.into_iter().map(WalletLedgerEntry::from).collect(),
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    pub async fn page_admin_ledger_entries(
        &self,
        request: PageSliceRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> StorageResult<Page<AdminWalletLedgerEntryResponse>> {
        let query = query::admin_query(filters, tz_offset_minutes);
        let total = ledger_entry_count(self, query.filtered_sql(), query.values()).await?;
        let rows = AdminLedgerEntryRow::find_by_statement(ledger_entry_statement(query, request))
            .all(self.database.connection())
            .await?;
        Ok(Page {
            items: rows.into_iter().map(AdminWalletLedgerEntryResponse::from).collect(),
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    pub async fn page_daily_usage_transactions(
        &self,
        wallet_id: &str,
        request: PageSliceRequest,
        detail: WalletDailyUsageDetailRequest,
    ) -> StorageResult<Page<WalletTransaction>> {
        let mut params = query::SqlParams::new();
        let filters = daily_usage_filters(&mut params, wallet_id, &detail);
        let total = daily_usage_count(self, &filters, params.values()).await?;
        let limit = params.push(pagination_value("limit", request.limit)?);
        let offset = params.push(pagination_value("offset", request.offset)?);
        let sql = format!(
            "SELECT t.* FROM wallet_transactions t WHERE {} ORDER BY t.created_at DESC, t.id DESC LIMIT {limit} OFFSET {offset}",
            filters.join(" AND ")
        );
        let rows = wallet_transaction_records::Model::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.into_values()))
            .all(self.database.connection())
            .await?;
        Ok(Page {
            items: rows.into_iter().map(WalletTransaction::from).collect(),
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }
}
