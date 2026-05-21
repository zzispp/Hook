use sea_orm::{DbBackend, FromQueryResult, Statement, Value};
use types::{
    pagination::PageSliceRequest,
    wallet::{WalletDailyUsageDetailRequest, WalletLedgerEntryFilters},
};

use crate::{StorageError, StorageResult};

use super::WalletStore;

const MODEL_USAGE_CATEGORY: &str = "consume";
const MODEL_USAGE_REASON: &str = "llm_model_usage";
const MODEL_USAGE_LINK_TYPE: &str = "llm_request_record";
const MODEL_USAGE_CONDITION: &str = "f.category = 'consume' AND f.reason_code = 'llm_model_usage' AND f.link_type = 'llm_request_record'";

pub(super) struct LedgerEntryQuery {
    filtered_sql: String,
    values: Vec<Value>,
}

impl LedgerEntryQuery {
    pub(super) fn filtered_sql(&self) -> &str {
        &self.filtered_sql
    }

    pub(super) fn values(&self) -> Vec<Value> {
        self.values.clone()
    }
}

pub(super) fn wallet_query(wallet_id: &str, filters: WalletLedgerEntryFilters, offset: i32) -> LedgerEntryQuery {
    let mut params = SqlParams::new();
    let mut clauses = vec![format!("t.wallet_id = {}", params.push(wallet_id.to_owned()))];
    add_entry_filters(&mut clauses, &mut params, filters, false);
    let offset = params.push(offset);
    LedgerEntryQuery {
        filtered_sql: filtered_sql(clauses, offset),
        values: params.into_values(),
    }
}

pub(super) fn admin_query(filters: WalletLedgerEntryFilters, offset: i32) -> LedgerEntryQuery {
    let mut params = SqlParams::new();
    let mut clauses = Vec::new();
    add_entry_filters(&mut clauses, &mut params, filters, true);
    let offset = params.push(offset);
    LedgerEntryQuery {
        filtered_sql: filtered_sql(clauses, offset),
        values: params.into_values(),
    }
}

pub(super) fn ledger_entry_statement(query: LedgerEntryQuery, request: PageSliceRequest) -> Statement {
    let mut params = SqlParams::from_values(query.values);
    let limit = params.push(pagination_value("limit", request.limit).expect("validated page limit must fit i64"));
    let offset = params.push(pagination_value("offset", request.offset).expect("validated page offset must fit i64"));
    let sql = format!(
        "{}, entries AS ({}) SELECT * FROM entries ORDER BY last_created_at DESC, id DESC LIMIT {limit} OFFSET {offset}",
        query.filtered_sql,
        entries_sql()
    );
    Statement::from_sql_and_values(DbBackend::Postgres, sql, params.into_values())
}

pub(super) async fn ledger_entry_count(store: &WalletStore, filtered: &str, values: Vec<Value>) -> StorageResult<u64> {
    let sql = format!("{filtered}, entries AS ({}) SELECT COUNT(*)::bigint AS total FROM entries", entries_sql());
    let row = CountRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .one(store.database.connection())
        .await?
        .ok_or_else(|| StorageError::Database("wallet ledger entry count returned no row".into()))?;
    non_negative_total(row.total)
}

pub(super) fn daily_usage_filters(params: &mut SqlParams, wallet_id: &str, request: &WalletDailyUsageDetailRequest) -> Vec<String> {
    vec![
        format!("t.wallet_id = {}", params.push(wallet_id.to_owned())),
        format!("t.category = {}", params.push(MODEL_USAGE_CATEGORY.to_owned())),
        format!("t.reason_code = {}", params.push(MODEL_USAGE_REASON.to_owned())),
        format!("t.link_type = {}", params.push(MODEL_USAGE_LINK_TYPE.to_owned())),
        format!(
            "to_char(date_trunc('day', ((t.created_at AT TIME ZONE 'UTC') + ({}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD') = {}",
            params.push(request.tz_offset_minutes),
            params.push(request.local_date.clone())
        ),
    ]
}

pub(super) async fn daily_usage_count(store: &WalletStore, filters: &[String], values: Vec<Value>) -> StorageResult<u64> {
    let sql = format!("SELECT COUNT(*)::bigint AS total FROM wallet_transactions t WHERE {}", filters.join(" AND "));
    let row = CountRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .one(store.database.connection())
        .await?
        .ok_or_else(|| StorageError::Database("wallet daily usage count returned no row".into()))?;
    non_negative_total(row.total)
}

pub(super) fn pagination_value(field: &str, value: u64) -> StorageResult<i64> {
    i64::try_from(value).map_err(|_| StorageError::Database(format!("wallet ledger {field} exceeds PostgreSQL integer range")))
}

#[derive(Default)]
pub(super) struct SqlParams {
    values: Vec<Value>,
}

impl SqlParams {
    pub(super) fn new() -> Self {
        Self { values: Vec::new() }
    }

    fn from_values(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub(super) fn push<T>(&mut self, value: T) -> String
    where
        T: Into<Value>,
    {
        self.values.push(value.into());
        format!("${}", self.values.len())
    }

    pub(super) fn values(&self) -> Vec<Value> {
        self.values.clone()
    }

    pub(super) fn into_values(self) -> Vec<Value> {
        self.values
    }
}

#[derive(Debug, FromQueryResult)]
struct CountRow {
    total: i64,
}

fn filtered_sql(clauses: Vec<String>, offset: String) -> String {
    format!(
        "WITH filtered AS (SELECT t.*, w.currency, w.status AS wallet_status, w.user_id, \
        COALESCE(u.username, w.user_id) AS owner_name, COALESCE(u.email, '') AS owner_email, 'user' AS owner_type, \
        to_char(date_trunc('day', ((t.created_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))), 'YYYY-MM-DD') AS local_date \
        FROM wallet_transactions t JOIN wallets w ON w.id = t.wallet_id LEFT JOIN users u ON u.id = w.user_id {})",
        where_clause(clauses)
    )
}

fn entries_sql() -> String {
    format!(
        "{} UNION ALL {}",
        transaction_entries_sql(),
        daily_model_usage_entries_sql()
    )
}

fn transaction_entries_sql() -> &'static str {
    "SELECT 'transaction' AS entry_kind, f.id, f.wallet_id, f.category, f.reason_code, f.amount, f.balance_before, f.balance_after, \
    f.recharge_balance_before, f.recharge_balance_after, f.gift_balance_before, f.gift_balance_after, f.link_type, f.link_id, f.operator_id, f.description, \
    f.created_at, NULL::text AS local_date, 1::bigint AS transaction_count, f.created_at AS first_created_at, f.created_at AS last_created_at, \
    f.currency, f.owner_name, f.owner_email, f.owner_type, f.wallet_status FROM filtered f \
    WHERE NOT (f.category = 'consume' AND f.reason_code = 'llm_model_usage' AND f.link_type = 'llm_request_record')"
}

fn daily_model_usage_entries_sql() -> String {
    format!(
        "SELECT 'daily_model_usage' AS entry_kind, CONCAT('daily_model_usage:', f.wallet_id, ':', f.local_date) AS id, f.wallet_id, 'consume' AS category, \
        'llm_model_usage' AS reason_code, COALESCE(SUM(f.amount), 0) AS amount, (ARRAY_AGG(f.balance_before ORDER BY f.created_at ASC, f.id ASC))[1] AS balance_before, \
        (ARRAY_AGG(f.balance_after ORDER BY f.created_at DESC, f.id DESC))[1] AS balance_after, \
        (ARRAY_AGG(f.recharge_balance_before ORDER BY f.created_at ASC, f.id ASC))[1] AS recharge_balance_before, \
        (ARRAY_AGG(f.recharge_balance_after ORDER BY f.created_at DESC, f.id DESC))[1] AS recharge_balance_after, \
        (ARRAY_AGG(f.gift_balance_before ORDER BY f.created_at ASC, f.id ASC))[1] AS gift_balance_before, \
        (ARRAY_AGG(f.gift_balance_after ORDER BY f.created_at DESC, f.id DESC))[1] AS gift_balance_after, \
        'llm_request_record' AS link_type, NULL::text AS link_id, NULL::text AS operator_id, NULL::text AS description, MAX(f.created_at) AS created_at, \
        f.local_date, COUNT(*)::bigint AS transaction_count, MIN(f.created_at) AS first_created_at, MAX(f.created_at) AS last_created_at, \
        MAX(f.currency) AS currency, MAX(f.owner_name) AS owner_name, MAX(f.owner_email) AS owner_email, MAX(f.owner_type) AS owner_type, \
        MAX(f.wallet_status) AS wallet_status FROM filtered f WHERE {MODEL_USAGE_CONDITION} GROUP BY f.wallet_id, f.local_date"
    )
}

fn add_entry_filters(clauses: &mut Vec<String>, params: &mut SqlParams, filters: WalletLedgerEntryFilters, include_owner: bool) {
    add_eq_filter(clauses, params, "t.category", filters.category);
    add_eq_filter(clauses, params, "t.reason_code", filters.reason_code);
    add_eq_filter(clauses, params, "t.link_type", filters.link_type);
    add_direction_filter(clauses, filters.direction.as_deref());
    add_balance_type_filter(clauses, filters.balance_type.as_deref());
    add_search_filter(clauses, params, filters.search, include_owner);
    add_owner_filter(clauses, filters.owner_type.as_deref());
}

fn add_eq_filter(clauses: &mut Vec<String>, params: &mut SqlParams, column: &str, value: Option<String>) {
    if let Some(value) = value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty()) {
        clauses.push(format!("{column} = {}", params.push(value)));
    }
}

fn add_direction_filter(clauses: &mut Vec<String>, value: Option<&str>) {
    match value {
        Some("income") => clauses.push("t.amount >= 0".into()),
        Some("expense") => clauses.push("t.amount < 0".into()),
        _ => {}
    }
}

fn add_balance_type_filter(clauses: &mut Vec<String>, value: Option<&str>) {
    match value {
        Some("recharge") => clauses.push("t.recharge_balance_before <> t.recharge_balance_after".into()),
        Some("gift") => clauses.push("t.gift_balance_before <> t.gift_balance_after".into()),
        _ => {}
    }
}

fn add_search_filter(clauses: &mut Vec<String>, params: &mut SqlParams, value: Option<String>, include_owner: bool) {
    let Some(value) = value.map(|item| item.trim().to_ascii_lowercase()).filter(|item| !item.is_empty()) else {
        return;
    };
    let placeholder = params.push(format!("%{value}%"));
    let mut columns = searchable_columns();
    if include_owner {
        columns.extend(owner_searchable_columns());
    }
    clauses.push(format!("({})", columns.into_iter().map(|column| format!("{column} LIKE {placeholder}")).collect::<Vec<_>>().join(" OR ")));
}

fn searchable_columns() -> Vec<&'static str> {
    vec![
        "LOWER(t.id)",
        "LOWER(t.category)",
        "LOWER(t.reason_code)",
        "LOWER(COALESCE(t.link_type, ''))",
        "LOWER(COALESCE(t.link_id, ''))",
        "LOWER(COALESCE(t.operator_id, ''))",
        "LOWER(COALESCE(t.description, ''))",
    ]
}

fn owner_searchable_columns() -> Vec<&'static str> {
    vec!["LOWER(w.id)", "LOWER(w.user_id)", "LOWER(COALESCE(u.username, ''))", "LOWER(COALESCE(u.email, ''))"]
}

fn add_owner_filter(clauses: &mut Vec<String>, value: Option<&str>) {
    if value == Some("user") {
        clauses.push("w.user_id IS NOT NULL".into());
    }
}

fn where_clause(clauses: Vec<String>) -> String {
    if clauses.is_empty() {
        return String::new();
    }
    format!("WHERE {}", clauses.join(" AND "))
}

fn non_negative_total(total: i64) -> StorageResult<u64> {
    u64::try_from(total).map_err(|_| StorageError::Database("wallet ledger count cannot be negative".into()))
}
