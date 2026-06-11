use rust_decimal::Decimal;
use sea_orm::{DbBackend, FromQueryResult, Statement, Value};
use time::format_description::well_known::Rfc3339;
use types::{
    pagination::{Page, PageSliceRequest},
    recharge::{RechargeOrderListFilters, RechargeOrderSummary, RechargeOrderSummaryPage, RechargeOrderUserSummary},
};

use crate::{StorageError, StorageResult};

use super::RechargeStore;

const SUMMARY_JOIN_SQL: &str = "FROM recharge_orders o JOIN users u ON u.id = o.user_id";

pub(super) async fn list_order_summary(
    store: &RechargeStore,
    request: PageSliceRequest,
    filters: RechargeOrderListFilters,
) -> StorageResult<RechargeOrderSummaryPage> {
    let filter = OrderSummaryFilter::from_filters(filters);
    let summary = summary_totals(store, &filter).await?;
    let total = user_total(store, &filter).await?;
    let items = user_rows(store, request, filter).await?;
    Ok(RechargeOrderSummaryPage {
        summary,
        users: Page {
            items,
            total,
            page: request.page,
            page_size: request.page_size,
        },
    })
}

async fn summary_totals(store: &RechargeStore, filter: &OrderSummaryFilter) -> StorageResult<RechargeOrderSummary> {
    let sql = format!(
        "SELECT COALESCE(SUM(o.payable_amount), 0) AS total_payable_amount, COUNT(*)::bigint AS order_count, \
        COUNT(DISTINCT o.user_id)::bigint AS user_count {SUMMARY_JOIN_SQL} {}",
        filter.where_sql()
    );
    SummaryTotalsRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, filter.values()))
        .one(store.database.connection())
        .await?
        .map(summary_from_row)
        .ok_or_else(|| StorageError::Database("recharge order summary returned no rows".into()))
}

async fn user_total(store: &RechargeStore, filter: &OrderSummaryFilter) -> StorageResult<u64> {
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM (SELECT o.user_id {SUMMARY_JOIN_SQL} {} GROUP BY o.user_id) users",
        filter.where_sql()
    );
    CountRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, filter.values()))
        .one(store.database.connection())
        .await?
        .map(|row| row.total.unwrap_or_default() as u64)
        .ok_or_else(|| StorageError::Database("recharge order summary user total returned no rows".into()))
}

async fn user_rows(store: &RechargeStore, request: PageSliceRequest, mut filter: OrderSummaryFilter) -> StorageResult<Vec<RechargeOrderUserSummary>> {
    let limit = filter.push((request.limit as i64).into());
    let offset = filter.push((request.offset as i64).into());
    let sql = format!(
        "SELECT o.user_id, u.username, u.email AS user_email, COUNT(*)::bigint AS order_count, \
        COALESCE(SUM(o.payable_amount), 0) AS total_payable_amount, MAX(o.paid_at) AS last_paid_at \
        {SUMMARY_JOIN_SQL} {} GROUP BY o.user_id, u.username, u.email \
        ORDER BY total_payable_amount DESC, last_paid_at DESC, o.user_id ASC LIMIT {limit} OFFSET {offset}",
        filter.where_sql()
    );
    UserSummaryRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, filter.values))
        .all(store.database.connection())
        .await
        .map(|rows| rows.into_iter().map(user_summary).collect())
        .map_err(Into::into)
}

struct OrderSummaryFilter {
    parts: Vec<String>,
    values: Vec<Value>,
}

impl OrderSummaryFilter {
    fn from_filters(filters: RechargeOrderListFilters) -> Self {
        let mut filter = Self {
            parts: Vec::new(),
            values: Vec::new(),
        };
        filter.add_status(filters.status.as_deref());
        filter.add_search(filters.search.as_deref());
        filter.add_paid_window(filters.paid_at_start, filters.paid_at_end);
        filter
    }

    fn add_status(&mut self, status: Option<&str>) {
        let Some(value) = normalized(status) else {
            return;
        };
        let placeholder = self.push(value.into());
        self.parts.push(format!("o.status = {placeholder}"));
    }

    fn add_search(&mut self, search: Option<&str>) {
        let Some(value) = normalized(search) else {
            return;
        };
        let placeholder = self.push(format!("%{value}%").into());
        self.parts.push(format!(
            "(o.order_no ILIKE {placeholder} OR o.package_name ILIKE {placeholder} OR u.username ILIKE {placeholder} OR u.email ILIKE {placeholder})"
        ));
    }

    fn add_paid_window(&mut self, started_at: Option<time::OffsetDateTime>, ended_at: Option<time::OffsetDateTime>) {
        if let Some(value) = started_at {
            let placeholder = self.push(value.into());
            self.parts.push(format!("o.paid_at >= {placeholder}"));
        }
        if let Some(value) = ended_at {
            let placeholder = self.push(value.into());
            self.parts.push(format!("o.paid_at < {placeholder}"));
        }
    }

    fn push(&mut self, value: Value) -> String {
        self.values.push(value);
        format!("${}", self.values.len())
    }

    fn values(&self) -> Vec<Value> {
        self.values.clone()
    }

    fn where_sql(&self) -> String {
        if self.parts.is_empty() {
            return String::new();
        }
        format!("WHERE {}", self.parts.join(" AND "))
    }
}

#[derive(Debug, FromQueryResult)]
struct CountRow {
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct SummaryTotalsRow {
    total_payable_amount: Option<Decimal>,
    order_count: Option<i64>,
    user_count: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct UserSummaryRow {
    user_id: String,
    username: String,
    user_email: String,
    order_count: Option<i64>,
    total_payable_amount: Option<Decimal>,
    last_paid_at: Option<time::OffsetDateTime>,
}

fn summary_from_row(row: SummaryTotalsRow) -> RechargeOrderSummary {
    RechargeOrderSummary {
        total_payable_amount: row.total_payable_amount.unwrap_or(Decimal::ZERO),
        order_count: row.order_count.unwrap_or_default() as u64,
        user_count: row.user_count.unwrap_or_default() as u64,
    }
}

fn user_summary(row: UserSummaryRow) -> RechargeOrderUserSummary {
    RechargeOrderUserSummary {
        user_id: row.user_id,
        username: row.username,
        user_email: row.user_email,
        order_count: row.order_count.unwrap_or_default() as u64,
        total_payable_amount: row.total_payable_amount.unwrap_or(Decimal::ZERO),
        last_paid_at: row.last_paid_at.map(format_timestamp),
    }
}

fn normalized(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned)
}

fn format_timestamp(value: time::OffsetDateTime) -> String {
    value.format(&Rfc3339).expect("recharge order summary timestamp must format as RFC3339")
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::recharge::RECHARGE_ORDER_STATUS_PAID;

    #[test]
    fn filter_adds_status_search_and_paid_window() {
        let filter = OrderSummaryFilter::from_filters(RechargeOrderListFilters {
            status: Some(RECHARGE_ORDER_STATUS_PAID.into()),
            search: Some("alice".into()),
            paid_at_start: Some(timestamp(1)),
            paid_at_end: Some(timestamp(2)),
            ..Default::default()
        });
        let where_sql = filter.where_sql();

        assert!(where_sql.contains("o.status = $1"), "{where_sql}");
        assert!(where_sql.contains("o.order_no ILIKE $2"), "{where_sql}");
        assert!(where_sql.contains("o.paid_at >= $3"), "{where_sql}");
        assert!(where_sql.contains("o.paid_at < $4"), "{where_sql}");
    }

    #[test]
    fn user_summary_sql_uses_required_aggregation_and_stable_order() {
        let request = PageSliceRequest {
            offset: 0,
            limit: 10,
            page: 1,
            page_size: 10,
        };
        let mut filter = OrderSummaryFilter::from_filters(RechargeOrderListFilters::default());
        let limit = filter.push((request.limit as i64).into());
        let offset = filter.push((request.offset as i64).into());
        let sql = format!(
            "SELECT o.user_id, u.username, u.email AS user_email, COUNT(*)::bigint AS order_count, \
            COALESCE(SUM(o.payable_amount), 0) AS total_payable_amount, MAX(o.paid_at) AS last_paid_at \
            {SUMMARY_JOIN_SQL} {} GROUP BY o.user_id, u.username, u.email \
            ORDER BY total_payable_amount DESC, last_paid_at DESC, o.user_id ASC LIMIT {limit} OFFSET {offset}",
            filter.where_sql()
        );

        assert!(sql.contains("SUM(o.payable_amount)"), "{sql}");
        assert!(sql.contains("GROUP BY o.user_id, u.username, u.email"), "{sql}");
        assert!(sql.contains("ORDER BY total_payable_amount DESC, last_paid_at DESC, o.user_id ASC"), "{sql}");
    }

    fn timestamp(days: i64) -> time::OffsetDateTime {
        time::OffsetDateTime::UNIX_EPOCH + time::Duration::days(days)
    }
}
