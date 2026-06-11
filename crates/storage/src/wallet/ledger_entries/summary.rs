use rust_decimal::Decimal;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use time::format_description::well_known::Rfc3339;
use types::{
    pagination::{Page, PageSliceRequest},
    wallet::{AdminWalletConsumptionSummaryItem, WalletLedgerEntryFilters},
};

use crate::{StorageError, StorageResult};

use super::{
    WalletStore,
    query::{SqlParams, add_date_range_filters, non_negative_total, pagination_value},
};

const CONSUME_CATEGORY: &str = "consume";

#[derive(Debug, FromQueryResult)]
struct ConsumptionSummaryRow {
    user_id: String,
    wallet_id: String,
    owner_name: String,
    owner_email: String,
    owner_type: String,
    wallet_status: String,
    currency: String,
    consumed_amount: Decimal,
    transaction_count: i64,
    first_created_at: time::OffsetDateTime,
    last_created_at: time::OffsetDateTime,
}

impl WalletStore {
    pub async fn page_admin_consumption_summary(
        &self,
        request: PageSliceRequest,
        filters: WalletLedgerEntryFilters,
    ) -> StorageResult<Page<AdminWalletConsumptionSummaryItem>> {
        let query = summary_query(filters);
        let total = summary_count(self, query.sql.as_str(), query.values()).await?;
        let rows = ConsumptionSummaryRow::find_by_statement(summary_statement(query, request))
            .all(self.database.connection())
            .await?;
        Ok(Page {
            items: rows
                .into_iter()
                .map(AdminWalletConsumptionSummaryItem::try_from)
                .collect::<StorageResult<Vec<_>>>()?,
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }
}

struct SummaryQuery {
    sql: String,
    values: Vec<sea_orm::Value>,
}

impl SummaryQuery {
    fn values(&self) -> Vec<sea_orm::Value> {
        self.values.clone()
    }
}

fn summary_query(filters: WalletLedgerEntryFilters) -> SummaryQuery {
    let mut params = SqlParams::new();
    let mut clauses = vec![format!("t.category = {}", params.push(CONSUME_CATEGORY.to_owned()))];
    add_date_range_filters(&mut clauses, &mut params, filters.date_range.as_ref());
    add_owner_filter(&mut clauses, filters.owner_type.as_deref());
    SummaryQuery {
        sql: summary_sql(clauses),
        values: params.into_values(),
    }
}

fn summary_statement(query: SummaryQuery, request: PageSliceRequest) -> Statement {
    let mut params = SqlParams::from_values(query.values);
    let limit = params.push(pagination_value("limit", request.limit).expect("validated page limit must fit i64"));
    let offset = params.push(pagination_value("offset", request.offset).expect("validated page offset must fit i64"));
    let sql = format!(
        "{} SELECT * FROM summary ORDER BY consumed_amount DESC, last_created_at DESC, user_id ASC LIMIT {limit} OFFSET {offset}",
        query.sql
    );
    Statement::from_sql_and_values(DbBackend::Postgres, sql, params.into_values())
}

async fn summary_count(store: &WalletStore, summary_sql: &str, values: Vec<sea_orm::Value>) -> StorageResult<u64> {
    let sql = format!("{summary_sql} SELECT COUNT(*)::bigint AS total FROM summary");
    let row = CountRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .one(store.database.connection())
        .await?
        .ok_or_else(|| StorageError::Database("wallet consumption summary count returned no row".into()))?;
    non_negative_total(row.total)
}

#[derive(Debug, FromQueryResult)]
struct CountRow {
    total: i64,
}

fn summary_sql(clauses: Vec<String>) -> String {
    format!(
        "WITH summary AS (SELECT w.user_id, w.id AS wallet_id, COALESCE(u.username, w.user_id) AS owner_name, \
        COALESCE(u.email, '') AS owner_email, 'user' AS owner_type, w.status AS wallet_status, w.currency, \
        COALESCE(-SUM(t.amount), 0) AS consumed_amount, COUNT(*)::bigint AS transaction_count, MIN(t.created_at) AS first_created_at, \
        MAX(t.created_at) AS last_created_at FROM wallet_transactions t JOIN wallets w ON w.id = t.wallet_id \
        LEFT JOIN users u ON u.id = w.user_id WHERE {} GROUP BY w.user_id, w.id, COALESCE(u.username, w.user_id), \
        COALESCE(u.email, ''), w.status, w.currency)",
        clauses.join(" AND ")
    )
}

fn add_owner_filter(clauses: &mut Vec<String>, value: Option<&str>) {
    if value == Some("user") {
        clauses.push("w.user_id IS NOT NULL".into());
    }
}

impl TryFrom<ConsumptionSummaryRow> for AdminWalletConsumptionSummaryItem {
    type Error = StorageError;

    fn try_from(row: ConsumptionSummaryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            user_id: row.user_id,
            wallet_id: row.wallet_id,
            owner_name: row.owner_name,
            owner_email: row.owner_email,
            owner_type: row.owner_type,
            wallet_status: row.wallet_status,
            currency: row.currency,
            consumed_amount: row.consumed_amount,
            transaction_count: row.transaction_count,
            first_created_at: format_timestamp(row.first_created_at)?,
            last_created_at: format_timestamp(row.last_created_at)?,
        })
    }
}

fn format_timestamp(value: time::OffsetDateTime) -> StorageResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| StorageError::Database(format!("wallet consumption summary timestamp format failed: {error}")))
}
