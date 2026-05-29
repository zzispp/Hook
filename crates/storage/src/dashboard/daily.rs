use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use types::{
    dashboard::{DashboardDailyModelSummary, DashboardDailyPeriod, DashboardDailyProviderSummary, DashboardDailyStat, DashboardDailyStats},
    pagination::{Page, PageRequest},
};

use crate::{StorageError, StorageResult};

use super::{
    DashboardStore, DashboardStoreOverviewQuery,
    daily_response::{DailyAggregateView, DailyMeasure, DailyResponseOptions, day_response, model_summary, provider_summary, sort_by_cost_name},
    scope::{SqlParams, scoped_time_where},
    token_context::sum_total_tokens_sql,
};

const EXCLUSIVE_END_OFFSET_SECONDS: i64 = 1;

pub(super) async fn daily_stats(store: &DashboardStore, query: &DashboardStoreOverviewQuery) -> StorageResult<DashboardDailyStats> {
    let rows = daily_rows(store, query).await?;
    let aggregates = DailyAggregates::from_rows(rows);
    let days = days(query, &aggregates.by_date)?;
    Ok(DashboardDailyStats {
        period: period(query)?,
        day_page: paged_days(&days, query.daily_page)?,
        days,
        model_summary: summaries(aggregates.models, query.include_admin_costs),
        provider_summary: provider_summaries(query, aggregates.providers),
    })
}

async fn daily_rows(store: &DashboardStore, query: &DashboardStoreOverviewQuery) -> StorageResult<Vec<DailyRow>> {
    let mut params = SqlParams::new();
    let offset = params.push(query.tz_offset_minutes);
    let where_sql = scoped_time_where(&query.scope, query.started_at, query.ended_at, &mut params);
    let sql = format!(
        "SELECT ((r.created_at AT TIME ZONE 'UTC') + ({offset}::int * INTERVAL '1 minute'))::date AS date, \
        COALESCE(r.model_name_snapshot, r.global_model_id, 'unknown') AS model_name, \
        COALESCE(r.provider_name_snapshot, r.provider_id, 'unknown') AS provider_name, \
        COUNT(*)::bigint AS request_count, \
        {} AS total_tokens, \
        COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
        COALESCE(SUM(COALESCE(r.upstream_total_cost, 0)), 0) AS upstream_total_cost, \
        COALESCE(SUM(r.total_latency_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL), 0)::double precision AS latency_total_ms, \
        COUNT(r.total_latency_ms) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL)::bigint AS latency_sample_count \
        FROM request_records r {where_sql} \
        GROUP BY date, model_name, provider_name \
        ORDER BY date ASC",
        sum_total_tokens_sql("r")
    );
    DailyRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .all(store.database().connection())
        .await
        .map_err(Into::into)
}

fn period(query: &DashboardStoreOverviewQuery) -> StorageResult<DashboardDailyPeriod> {
    let start_date = local_date(query.started_at, query.tz_offset_minutes)?;
    let end_date = local_date(query.ended_at - time::Duration::seconds(EXCLUSIVE_END_OFFSET_SECONDS), query.tz_offset_minutes)?;
    Ok(DashboardDailyPeriod {
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        days: inclusive_day_count(start_date, end_date),
    })
}

fn days(query: &DashboardStoreOverviewQuery, by_date: &BTreeMap<time::Date, DailyAggregate>) -> StorageResult<Vec<DashboardDailyStat>> {
    let start_date = local_date(query.started_at, query.tz_offset_minutes)?;
    let end_date = local_date(query.ended_at - time::Duration::seconds(EXCLUSIVE_END_OFFSET_SECONDS), query.tz_offset_minutes)?;
    let options = DailyResponseOptions {
        include_admin_breakdowns: query.include_admin_breakdowns,
        include_admin_costs: query.include_admin_costs,
    };
    let mut date = start_date;
    let mut output = Vec::new();
    while date <= end_date {
        output.push(day_response(date, by_date.get(&date), options));
        let Some(next_date) = date.next_day() else {
            return Err(StorageError::Database("dashboard daily date overflow".into()));
        };
        date = next_date;
    }
    Ok(output)
}

fn paged_days(days: &[DashboardDailyStat], request: PageRequest) -> StorageResult<Page<DashboardDailyStat>> {
    let total = days.len() as u64;
    let start = page_start(request)?;
    let limit = page_limit(request)?;
    Ok(Page {
        items: days.iter().rev().skip(start).take(limit).cloned().collect(),
        total,
        page: request.page,
        page_size: request.page_size,
    })
}

fn page_start(request: PageRequest) -> StorageResult<usize> {
    let offset = request
        .page
        .checked_sub(1)
        .and_then(|page| page.checked_mul(request.page_size))
        .ok_or_else(|| StorageError::Database("dashboard daily page offset overflow".into()))?;
    usize::try_from(offset).map_err(|_| StorageError::Database("dashboard daily page offset exceeds supported size".into()))
}

fn page_limit(request: PageRequest) -> StorageResult<usize> {
    usize::try_from(request.page_size).map_err(|_| StorageError::Database("dashboard daily page size exceeds supported size".into()))
}

fn summaries(values: BTreeMap<String, DailyMeasure>, include_admin_costs: bool) -> Vec<DashboardDailyModelSummary> {
    let mut output = values
        .into_iter()
        .map(|(name, value)| model_summary(name, value, include_admin_costs))
        .collect::<Vec<_>>();
    sort_by_cost_name(&mut output);
    output
}

fn provider_summaries(query: &DashboardStoreOverviewQuery, values: BTreeMap<String, DailyMeasure>) -> Vec<DashboardDailyProviderSummary> {
    if !query.include_admin_breakdowns {
        return Vec::new();
    }
    let mut output = values
        .into_iter()
        .map(|(name, value)| provider_summary(name, value, query.include_admin_costs))
        .collect::<Vec<_>>();
    sort_by_cost_name(&mut output);
    output
}

fn local_date(value: time::OffsetDateTime, offset_minutes: i32) -> StorageResult<time::Date> {
    let seconds = offset_minutes
        .checked_mul(60)
        .ok_or_else(|| StorageError::Database("dashboard timezone offset exceeds supported range".into()))?;
    let offset = time::UtcOffset::from_whole_seconds(seconds).map_err(|_| StorageError::Database("dashboard timezone offset is out of range".into()))?;
    Ok(value.to_offset(offset).date())
}

fn inclusive_day_count(start_date: time::Date, end_date: time::Date) -> usize {
    usize::try_from((end_date - start_date).whole_days() + 1).unwrap_or_default()
}

#[derive(Default)]
struct DailyAggregates {
    by_date: BTreeMap<time::Date, DailyAggregate>,
    models: BTreeMap<String, DailyMeasure>,
    providers: BTreeMap<String, DailyMeasure>,
}

impl DailyAggregates {
    fn from_rows(rows: Vec<DailyRow>) -> Self {
        let mut output = Self::default();
        for row in rows {
            output.record(row);
        }
        output
    }

    fn record(&mut self, row: DailyRow) {
        self.by_date.entry(row.date).or_default().record(&row);
        self.models.entry(row.model_name.clone()).or_default().record(&row);
        self.providers.entry(row.provider_name.clone()).or_default().record(&row);
    }
}

#[derive(Default)]
struct DailyAggregate {
    measure: DailyMeasure,
    models: BTreeMap<String, DailyMeasure>,
    providers: BTreeMap<String, DailyMeasure>,
}

impl DailyAggregate {
    fn record(&mut self, row: &DailyRow) {
        self.measure.record(row);
        self.models.entry(row.model_name.clone()).or_default().record(row);
        self.providers.entry(row.provider_name.clone()).or_default().record(row);
    }
}

impl DailyMeasure {
    fn record(&mut self, row: &DailyRow) {
        self.request_count += row.request_count.unwrap_or_default();
        self.total_tokens += row.total_tokens.unwrap_or_default();
        self.total_cost += row.total_cost.unwrap_or(Decimal::ZERO);
        self.upstream_total_cost += row.upstream_total_cost.unwrap_or(Decimal::ZERO);
        self.latency_total_ms += row.latency_total_ms.unwrap_or_default();
        self.latency_sample_count += row.latency_sample_count.unwrap_or_default();
    }
}

#[derive(Debug, FromQueryResult)]
struct DailyRow {
    date: time::Date,
    model_name: String,
    provider_name: String,
    request_count: Option<i64>,
    total_tokens: Option<i64>,
    total_cost: Option<Decimal>,
    upstream_total_cost: Option<Decimal>,
    latency_total_ms: Option<f64>,
    latency_sample_count: Option<i64>,
}

impl DailyAggregateView for DailyAggregate {
    fn measure(&self) -> &DailyMeasure {
        &self.measure
    }

    fn models(&self) -> &BTreeMap<String, DailyMeasure> {
        &self.models
    }

    fn providers(&self) -> &BTreeMap<String, DailyMeasure> {
        &self.providers
    }
}
