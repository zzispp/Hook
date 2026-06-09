use std::collections::HashMap;

use rust_decimal::Decimal;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use types::dashboard::{DashboardActivityDay, DashboardActivityResponse};

use crate::StorageResult;

use super::{
    DashboardStore, DashboardStoreActivityQuery,
    money::admin_cost_metrics,
    scope::{SqlParams, scope_response, scoped_metric_bucket_where},
};

const ACTIVITY_GRANULARITY: &str = "hour";

pub(super) async fn activity(store: &DashboardStore, query: DashboardStoreActivityQuery) -> StorageResult<DashboardActivityResponse> {
    let rows = activity_rows(store, &query).await?;
    let days = fill_days(query.start_date, query.end_date, rows, query.include_admin_costs);
    let max_request_count = days.iter().map(|day| day.request_count).max().unwrap_or_default();
    Ok(DashboardActivityResponse {
        scope: scope_response(&query.scope),
        start_date: query.start_date.to_string(),
        end_date: query.end_date.to_string(),
        total_days: days.len(),
        max_request_count,
        days,
    })
}

async fn activity_rows(store: &DashboardStore, query: &DashboardStoreActivityQuery) -> StorageResult<Vec<ActivityRow>> {
    let mut params = SqlParams::new();
    let offset = params.push(query.tz_offset_minutes);
    let where_sql = scoped_metric_bucket_where(&query.scope, query.started_at, query.ended_at, ACTIVITY_GRANULARITY, &mut params);
    let sql = format!(
        "SELECT ((b.bucket_started_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))::date AS date, \
        COALESCE(SUM(b.request_count), 0)::bigint AS request_count, \
        COALESCE(SUM(b.total_tokens), 0)::bigint AS total_tokens, \
        COALESCE(SUM(b.total_cost), 0) AS total_cost, \
        COALESCE(SUM(b.base_cost), 0) AS base_cost, \
        COALESCE(SUM(b.upstream_total_cost), 0) AS upstream_total_cost \
        FROM dashboard_request_metric_buckets b {where_sql} \
        GROUP BY date \
        ORDER BY date ASC"
    );
    ActivityRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await
        .map_err(Into::into)
}

pub(crate) fn fill_days(start_date: time::Date, end_date: time::Date, rows: Vec<ActivityRow>, include_admin_costs: bool) -> Vec<DashboardActivityDay> {
    let mapped = rows.into_iter().map(|row| (row.date, row)).collect::<HashMap<time::Date, ActivityRow>>();
    let mut days = Vec::new();
    let mut date = start_date;
    while date <= end_date {
        days.push(day_response(date, mapped.get(&date), include_admin_costs));
        date = date.next_day().expect("dashboard activity date should advance");
    }
    days
}

fn day_response(date: time::Date, row: Option<&ActivityRow>, include_admin_costs: bool) -> DashboardActivityDay {
    let total_cost = row.and_then(|value| value.total_cost).unwrap_or(Decimal::ZERO);
    let metrics = admin_cost_metrics(total_cost, upstream_cost(row), include_admin_costs);
    DashboardActivityDay {
        date: date.to_string(),
        request_count: row.and_then(|value| value.request_count).unwrap_or_default(),
        total_tokens: row.and_then(|value| value.total_tokens).unwrap_or_default(),
        total_cost,
        base_cost: row.and_then(|value| value.base_cost).unwrap_or(Decimal::ZERO),
        upstream_total_cost: metrics.upstream_total_cost,
        profit: metrics.profit,
        profit_rate: metrics.profit_rate,
    }
}

fn upstream_cost(row: Option<&ActivityRow>) -> Decimal {
    row.and_then(|value| value.upstream_total_cost).unwrap_or(Decimal::ZERO)
}

#[derive(Clone, Debug, FromQueryResult)]
pub(crate) struct ActivityRow {
    pub date: time::Date,
    pub request_count: Option<i64>,
    pub total_tokens: Option<i64>,
    pub total_cost: Option<Decimal>,
    pub base_cost: Option<Decimal>,
    pub upstream_total_cost: Option<Decimal>,
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::{ActivityRow, fill_days};

    #[test]
    fn fill_days_adds_zero_rows_and_preserves_values() {
        let start = time::Date::from_calendar_date(2026, time::Month::May, 16).unwrap();
        let end = time::Date::from_calendar_date(2026, time::Month::May, 18).unwrap();
        let rows = vec![ActivityRow {
            date: time::Date::from_calendar_date(2026, time::Month::May, 17).unwrap(),
            request_count: Some(3),
            total_tokens: Some(42),
            total_cost: Some(Decimal::new(12, 2)),
            base_cost: Some(Decimal::new(7, 2)),
            upstream_total_cost: Some(Decimal::new(3, 2)),
        }];

        let days = fill_days(start, end, rows, true);

        assert_eq!(days.len(), 3);
        assert_eq!(days[0].request_count, 0);
        assert_eq!(days[1].request_count, 3);
        assert_eq!(days[1].total_tokens, 42);
        assert_eq!(days[1].base_cost, Decimal::new(7, 2));
        assert_eq!(days[1].upstream_total_cost, Decimal::new(3, 2));
        assert_eq!(days[1].profit, Decimal::new(9, 2));
        assert_eq!(days[1].profit_rate, 0.75);
        assert_eq!(days[2].total_cost, Decimal::ZERO);
        assert_eq!(days[2].base_cost, Decimal::ZERO);
        assert_eq!(days[2].upstream_total_cost, Decimal::ZERO);
    }

    #[test]
    fn fill_days_hides_admin_costs_when_requested() {
        let date = time::Date::from_calendar_date(2026, time::Month::May, 17).unwrap();
        let rows = vec![ActivityRow {
            date,
            request_count: Some(3),
            total_tokens: Some(42),
            total_cost: Some(Decimal::new(12, 2)),
            base_cost: Some(Decimal::new(7, 2)),
            upstream_total_cost: Some(Decimal::new(3, 2)),
        }];

        let days = fill_days(date, date, rows, false);

        assert_eq!(days[0].total_cost, Decimal::new(12, 2));
        assert_eq!(days[0].upstream_total_cost, Decimal::ZERO);
        assert_eq!(days[0].profit, Decimal::ZERO);
        assert_eq!(days[0].profit_rate, 0.0);
    }
}
