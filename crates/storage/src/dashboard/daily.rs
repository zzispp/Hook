use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use types::{
    dashboard::{
        DashboardDailyBreakdownItem, DashboardDailyModelSummary, DashboardDailyPeriod, DashboardDailyProviderSummary, DashboardDailyStat, DashboardDailyStats,
    },
    pagination::{Page, PageRequest},
};

use crate::{StorageError, StorageResult};

use super::{
    DashboardStore, DashboardStoreOverviewQuery,
    scope::{SqlParams, scoped_time_where},
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
        model_summary: summaries(aggregates.models),
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
        COALESCE(SUM(COALESCE(r.total_tokens, COALESCE(r.prompt_tokens, 0) + COALESCE(r.completion_tokens, 0), 0)), 0)::bigint AS total_tokens, \
        COALESCE(SUM(COALESCE(r.total_cost, 0)), 0) AS total_cost, \
        COALESCE(SUM(r.total_latency_ms::double precision) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL), 0)::double precision AS latency_total_ms, \
        COUNT(r.total_latency_ms) FILTER (WHERE r.status IN ('success', 'failed', 'cancelled') AND r.total_latency_ms IS NOT NULL)::bigint AS latency_sample_count \
        FROM request_records r {where_sql} \
        GROUP BY date, model_name, provider_name \
        ORDER BY date ASC"
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
    let mut date = start_date;
    let mut output = Vec::new();
    while date <= end_date {
        output.push(day_response(date, by_date.get(&date), query.include_admin_breakdowns));
        let Some(next_date) = date.next_day() else {
            return Err(StorageError::Database("dashboard daily date overflow".into()));
        };
        date = next_date;
    }
    Ok(output)
}

fn day_response(date: time::Date, aggregate: Option<&DailyAggregate>, include_admin_breakdowns: bool) -> DashboardDailyStat {
    let empty = DailyAggregate::default();
    let value = aggregate.unwrap_or(&empty);
    DashboardDailyStat {
        date: date.to_string(),
        request_count: value.measure.request_count,
        total_tokens: value.measure.total_tokens,
        total_cost: value.measure.total_cost,
        avg_latency_ms: avg_latency(value.measure.latency_total_ms, value.measure.latency_sample_count),
        unique_models: value.models.len(),
        unique_providers: if include_admin_breakdowns { value.providers.len() } else { 0 },
        model_breakdown: model_breakdown(&value.models),
    }
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

fn model_breakdown(models: &BTreeMap<String, DailyMeasure>) -> Vec<DashboardDailyBreakdownItem> {
    let mut items = models
        .iter()
        .map(|(name, value)| DashboardDailyBreakdownItem {
            name: name.clone(),
            request_count: value.request_count,
            total_tokens: value.total_tokens,
            total_cost: value.total_cost,
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.total_cost.cmp(&left.total_cost).then_with(|| left.name.cmp(&right.name)));
    items
}

fn summaries(values: BTreeMap<String, DailyMeasure>) -> Vec<DashboardDailyModelSummary> {
    let mut output = values
        .into_iter()
        .map(|(name, value)| DashboardDailyModelSummary {
            name,
            request_count: value.request_count,
            total_tokens: value.total_tokens,
            total_cost: value.total_cost,
            avg_latency_ms: avg_latency(value.latency_total_ms, value.latency_sample_count),
            cost_per_request: cost_per_request(value.total_cost, value.request_count),
            tokens_per_request: tokens_per_request(value.total_tokens, value.request_count),
        })
        .collect::<Vec<_>>();
    output.sort_by(|left, right| right.total_cost.cmp(&left.total_cost).then_with(|| left.name.cmp(&right.name)));
    output
}

fn provider_summaries(query: &DashboardStoreOverviewQuery, values: BTreeMap<String, DailyMeasure>) -> Vec<DashboardDailyProviderSummary> {
    if !query.include_admin_breakdowns {
        return Vec::new();
    }
    let mut output = values
        .into_iter()
        .map(|(name, value)| DashboardDailyProviderSummary {
            name,
            request_count: value.request_count,
            total_tokens: value.total_tokens,
            total_cost: value.total_cost,
        })
        .collect::<Vec<_>>();
    output.sort_by(|left, right| right.total_cost.cmp(&left.total_cost).then_with(|| left.name.cmp(&right.name)));
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

fn avg_latency(total_ms: f64, samples: i64) -> Option<f64> {
    if samples <= 0 {
        return None;
    }
    Some(total_ms / samples as f64)
}

fn cost_per_request(total_cost: Decimal, request_count: i64) -> Decimal {
    if request_count <= 0 {
        return Decimal::ZERO;
    }
    total_cost / Decimal::from(request_count)
}

fn tokens_per_request(total_tokens: i64, request_count: i64) -> f64 {
    if request_count <= 0 {
        return 0.0;
    }
    total_tokens as f64 / request_count as f64
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

#[derive(Default)]
struct DailyMeasure {
    request_count: i64,
    total_tokens: i64,
    total_cost: Decimal,
    latency_total_ms: f64,
    latency_sample_count: i64,
}

impl DailyMeasure {
    fn record(&mut self, row: &DailyRow) {
        self.request_count += row.request_count.unwrap_or_default();
        self.total_tokens += row.total_tokens.unwrap_or_default();
        self.total_cost += row.total_cost.unwrap_or(Decimal::ZERO);
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
    latency_total_ms: Option<f64>,
    latency_sample_count: Option<i64>,
}
