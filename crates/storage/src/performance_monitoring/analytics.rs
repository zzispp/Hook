use sea_orm::{DbBackend, FromQueryResult, Statement, Value};
use types::performance_monitoring::{EffectiveTimeRange, PerformanceMonitoringAnalyticsRequest, PerformanceMonitoringAnalyticsResponse, UpstreamPerformance};

use crate::StorageResult;

use super::{
    PerformanceMonitoringStore,
    analytics_rows::{
        ErrorDistributionRow, ErrorTrendRow, PercentileRow, RecentErrorRow, UpstreamProviderRow, UpstreamSummaryRow, UpstreamTimelineRow, group_error_trend,
    },
    analytics_sql::{
        DEFAULT_ANALYTICS_LIMIT, DEFAULT_SLOW_THRESHOLD_MS, MAX_ANALYTICS_LIMIT, RECENT_ERROR_LIMIT, UpstreamFilters, error_distribution_sql, error_trend_sql,
        percentile_sql, plan_values, recent_errors_sql, upstream_providers_sql, upstream_summary_sql, upstream_timeline_sql,
    },
    query,
    types::SnapshotQueryPlan,
};

pub(super) async fn analytics(
    store: &PerformanceMonitoringStore,
    request: PerformanceMonitoringAnalyticsRequest,
    now: time::OffsetDateTime,
) -> StorageResult<PerformanceMonitoringAnalyticsResponse> {
    let plan = query::range_plan(request.range, now);
    let filters = UpstreamFilters::from_request(&request);
    let limit = request.limit.unwrap_or(DEFAULT_ANALYTICS_LIMIT).clamp(1, MAX_ANALYTICS_LIMIT);
    let slow_threshold_ms = request.slow_threshold_ms.unwrap_or(DEFAULT_SLOW_THRESHOLD_MS).max(1);
    Ok(PerformanceMonitoringAnalyticsResponse {
        range: request.range,
        effective_range: effective_range(&plan),
        bucket_granularity: plan.granularity,
        percentiles: percentiles(store, &plan).await?,
        error_distribution: error_distribution(store, &plan).await?,
        error_trend: error_trend(store, &plan).await?,
        upstream_performance: upstream_performance(store, &plan, &filters, limit, slow_threshold_ms).await?,
        recent_errors: recent_errors(store, &plan).await?,
    })
}

fn effective_range(plan: &SnapshotQueryPlan) -> EffectiveTimeRange {
    EffectiveTimeRange {
        started_at: query::format_timestamp(plan.started_at),
        ended_at: query::format_timestamp(plan.ended_at),
    }
}

async fn percentiles(
    store: &PerformanceMonitoringStore,
    plan: &SnapshotQueryPlan,
) -> StorageResult<Vec<types::performance_monitoring::PerformancePercentilePoint>> {
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, percentile_sql(plan.granularity), plan_values(plan));
    let rows = PercentileRow::find_by_statement(statement).all(store.connection()).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

async fn error_distribution(
    store: &PerformanceMonitoringStore,
    plan: &SnapshotQueryPlan,
) -> StorageResult<Vec<types::performance_monitoring::ErrorDistributionItem>> {
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, error_distribution_sql(), plan_values(plan));
    let rows = ErrorDistributionRow::find_by_statement(statement).all(store.connection()).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

async fn error_trend(store: &PerformanceMonitoringStore, plan: &SnapshotQueryPlan) -> StorageResult<Vec<types::performance_monitoring::ErrorTrendPoint>> {
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, error_trend_sql(plan.granularity), plan_values(plan));
    let rows = ErrorTrendRow::find_by_statement(statement).all(store.connection()).await?;
    Ok(group_error_trend(rows, plan.granularity))
}

async fn upstream_performance(
    store: &PerformanceMonitoringStore,
    plan: &SnapshotQueryPlan,
    filters: &UpstreamFilters,
    limit: usize,
    slow_threshold_ms: i64,
) -> StorageResult<UpstreamPerformance> {
    let summary = upstream_summary(store, plan, filters, slow_threshold_ms).await?;
    let providers = upstream_providers(store, plan, filters, limit, slow_threshold_ms).await?;
    let provider_ids = providers.iter().map(|item| item.provider_id.clone()).collect::<Vec<_>>();
    let timeline = upstream_timeline(store, plan, filters, &provider_ids, slow_threshold_ms).await?;
    Ok(UpstreamPerformance { summary, providers, timeline })
}

async fn upstream_summary(
    store: &PerformanceMonitoringStore,
    plan: &SnapshotQueryPlan,
    filters: &UpstreamFilters,
    slow_threshold_ms: i64,
) -> StorageResult<types::performance_monitoring::UpstreamPerformanceSummary> {
    let parts = upstream_summary_sql(plan, filters, slow_threshold_ms);
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, parts.sql, parts.values);
    let row = UpstreamSummaryRow::find_by_statement(statement).one(store.connection()).await?;
    Ok(row.unwrap_or_default().into())
}

async fn upstream_providers(
    store: &PerformanceMonitoringStore,
    plan: &SnapshotQueryPlan,
    filters: &UpstreamFilters,
    limit: usize,
    slow_threshold_ms: i64,
) -> StorageResult<Vec<types::performance_monitoring::UpstreamPerformanceProvider>> {
    let parts = upstream_providers_sql(plan, filters, limit, slow_threshold_ms);
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, parts.sql, parts.values);
    let rows = UpstreamProviderRow::find_by_statement(statement).all(store.connection()).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

async fn upstream_timeline(
    store: &PerformanceMonitoringStore,
    plan: &SnapshotQueryPlan,
    filters: &UpstreamFilters,
    provider_ids: &[String],
    slow_threshold_ms: i64,
) -> StorageResult<Vec<types::performance_monitoring::UpstreamPerformanceTimelinePoint>> {
    if provider_ids.is_empty() {
        return Ok(Vec::new());
    }
    let parts = upstream_timeline_sql(plan, filters, provider_ids, slow_threshold_ms);
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, parts.sql, parts.values);
    let rows = UpstreamTimelineRow::find_by_statement(statement).all(store.connection()).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

async fn recent_errors(
    store: &PerformanceMonitoringStore,
    plan: &SnapshotQueryPlan,
) -> StorageResult<Vec<types::performance_monitoring::RecentPerformanceError>> {
    let values = vec![Value::from(plan.started_at), Value::from(plan.ended_at), Value::from(RECENT_ERROR_LIMIT)];
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, recent_errors_sql(), values);
    let rows = RecentErrorRow::find_by_statement(statement).all(store.connection()).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}
