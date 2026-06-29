use std::collections::BTreeMap;

use sea_orm::FromQueryResult;
use types::performance_monitoring::{
    ErrorDistributionItem, ErrorTrendPoint, PerformancePercentilePoint, RecentPerformanceError, SnapshotGranularity, UpstreamPerformanceProvider,
    UpstreamPerformanceSummary, UpstreamPerformanceTimelinePoint,
};

use super::query;

#[derive(Debug, FromQueryResult)]
pub(super) struct PercentileRow {
    pub bucket_started_at: time::OffsetDateTime,
    pub bucket_ended_at: time::OffsetDateTime,
    pub p50_latency_ms: Option<i64>,
    pub p90_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
    pub p50_ttfb_ms: Option<i64>,
    pub p90_ttfb_ms: Option<i64>,
    pub p99_ttfb_ms: Option<i64>,
    pub p50_response_headers_ms: Option<i64>,
    pub p90_response_headers_ms: Option<i64>,
    pub p99_response_headers_ms: Option<i64>,
    pub p50_first_output_ms: Option<i64>,
    pub p90_first_output_ms: Option<i64>,
    pub p99_first_output_ms: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct ErrorDistributionRow {
    pub category: String,
    pub count: i64,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct ErrorTrendRow {
    pub bucket_started_at: time::OffsetDateTime,
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Default, FromQueryResult)]
pub(super) struct UpstreamSummaryRow {
    pub request_count: Option<i64>,
    pub success_count: Option<i64>,
    pub error_count: Option<i64>,
    pub output_tokens: Option<i64>,
    pub avg_output_tps: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub p90_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
    pub p90_ttfb_ms: Option<i64>,
    pub p99_ttfb_ms: Option<i64>,
    pub p90_response_headers_ms: Option<i64>,
    pub p99_response_headers_ms: Option<i64>,
    pub p90_first_output_ms: Option<i64>,
    pub p99_first_output_ms: Option<i64>,
    pub tps_sample_count: Option<i64>,
    pub latency_sample_count: Option<i64>,
    pub ttfb_sample_count: Option<i64>,
    pub response_headers_sample_count: Option<i64>,
    pub first_output_sample_count: Option<i64>,
    pub slow_request_count: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct UpstreamProviderRow {
    pub provider_id: String,
    pub provider_name: String,
    pub request_count: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub output_tokens: i64,
    pub avg_output_tps: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub p90_latency_ms: Option<i64>,
    pub p99_latency_ms: Option<i64>,
    pub p90_ttfb_ms: Option<i64>,
    pub p99_ttfb_ms: Option<i64>,
    pub p90_response_headers_ms: Option<i64>,
    pub p99_response_headers_ms: Option<i64>,
    pub p90_first_output_ms: Option<i64>,
    pub p99_first_output_ms: Option<i64>,
    pub tps_sample_count: i64,
    pub latency_sample_count: i64,
    pub ttfb_sample_count: i64,
    pub response_headers_sample_count: i64,
    pub first_output_sample_count: i64,
    pub slow_request_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct UpstreamTimelineRow {
    pub bucket_started_at: time::OffsetDateTime,
    pub bucket_ended_at: time::OffsetDateTime,
    pub provider_id: String,
    pub provider_name: String,
    pub request_count: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub output_tokens: i64,
    pub avg_output_tps: Option<f64>,
    pub avg_ttfb_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub avg_response_headers_ms: Option<f64>,
    pub avg_first_output_ms: Option<f64>,
    pub slow_request_count: i64,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct RecentErrorRow {
    pub created_at: time::OffsetDateTime,
    pub request_id: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub model: Option<String>,
    pub status_code: Option<i32>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub response_headers_ms: Option<i64>,
    pub first_output_ms: Option<i64>,
    pub latency_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
}

impl From<PercentileRow> for PerformancePercentilePoint {
    fn from(row: PercentileRow) -> Self {
        Self {
            bucket_started_at: query::format_timestamp(row.bucket_started_at),
            bucket_ended_at: query::format_timestamp(row.bucket_ended_at),
            p50_latency_ms: row.p50_latency_ms,
            p90_latency_ms: row.p90_latency_ms,
            p99_latency_ms: row.p99_latency_ms,
            p50_ttfb_ms: row.p50_ttfb_ms,
            p90_ttfb_ms: row.p90_ttfb_ms,
            p99_ttfb_ms: row.p99_ttfb_ms,
            p50_response_headers_ms: row.p50_response_headers_ms,
            p90_response_headers_ms: row.p90_response_headers_ms,
            p99_response_headers_ms: row.p99_response_headers_ms,
            p50_first_output_ms: row.p50_first_output_ms,
            p90_first_output_ms: row.p90_first_output_ms,
            p99_first_output_ms: row.p99_first_output_ms,
        }
    }
}

impl From<ErrorDistributionRow> for ErrorDistributionItem {
    fn from(row: ErrorDistributionRow) -> Self {
        Self {
            category: row.category,
            count: row.count,
        }
    }
}

impl From<UpstreamSummaryRow> for UpstreamPerformanceSummary {
    fn from(row: UpstreamSummaryRow) -> Self {
        let request_count = row.request_count.unwrap_or_default();
        let success_count = row.success_count.unwrap_or_default();
        let error_count = row.error_count.unwrap_or_default();
        Self {
            request_count,
            success_count,
            error_count,
            success_rate: ratio(success_count, request_count),
            error_rate: ratio(error_count, request_count),
            output_tokens: row.output_tokens.unwrap_or_default(),
            avg_output_tps: row.avg_output_tps,
            avg_ttfb_ms: row.avg_ttfb_ms,
            avg_latency_ms: row.avg_latency_ms,
            avg_response_headers_ms: row.avg_response_headers_ms,
            avg_first_output_ms: row.avg_first_output_ms,
            p90_latency_ms: row.p90_latency_ms,
            p99_latency_ms: row.p99_latency_ms,
            p90_ttfb_ms: row.p90_ttfb_ms,
            p99_ttfb_ms: row.p99_ttfb_ms,
            p90_response_headers_ms: row.p90_response_headers_ms,
            p99_response_headers_ms: row.p99_response_headers_ms,
            p90_first_output_ms: row.p90_first_output_ms,
            p99_first_output_ms: row.p99_first_output_ms,
            tps_sample_count: row.tps_sample_count.unwrap_or_default(),
            latency_sample_count: row.latency_sample_count.unwrap_or_default(),
            ttfb_sample_count: row.ttfb_sample_count.unwrap_or_default(),
            response_headers_sample_count: row.response_headers_sample_count.unwrap_or_default(),
            first_output_sample_count: row.first_output_sample_count.unwrap_or_default(),
            slow_request_count: row.slow_request_count.unwrap_or_default(),
        }
    }
}

impl From<UpstreamProviderRow> for UpstreamPerformanceProvider {
    fn from(row: UpstreamProviderRow) -> Self {
        Self {
            success_rate: ratio(row.success_count, row.request_count),
            error_rate: ratio(row.error_count, row.request_count),
            provider_id: row.provider_id,
            provider_name: row.provider_name,
            request_count: row.request_count,
            success_count: row.success_count,
            error_count: row.error_count,
            output_tokens: row.output_tokens,
            avg_output_tps: row.avg_output_tps,
            avg_ttfb_ms: row.avg_ttfb_ms,
            avg_latency_ms: row.avg_latency_ms,
            avg_response_headers_ms: row.avg_response_headers_ms,
            avg_first_output_ms: row.avg_first_output_ms,
            p90_latency_ms: row.p90_latency_ms,
            p99_latency_ms: row.p99_latency_ms,
            p90_ttfb_ms: row.p90_ttfb_ms,
            p99_ttfb_ms: row.p99_ttfb_ms,
            p90_response_headers_ms: row.p90_response_headers_ms,
            p99_response_headers_ms: row.p99_response_headers_ms,
            p90_first_output_ms: row.p90_first_output_ms,
            p99_first_output_ms: row.p99_first_output_ms,
            tps_sample_count: row.tps_sample_count,
            latency_sample_count: row.latency_sample_count,
            ttfb_sample_count: row.ttfb_sample_count,
            response_headers_sample_count: row.response_headers_sample_count,
            first_output_sample_count: row.first_output_sample_count,
            slow_request_count: row.slow_request_count,
        }
    }
}

impl From<UpstreamTimelineRow> for UpstreamPerformanceTimelinePoint {
    fn from(row: UpstreamTimelineRow) -> Self {
        Self {
            bucket_started_at: query::format_timestamp(row.bucket_started_at),
            bucket_ended_at: query::format_timestamp(row.bucket_ended_at),
            success_rate: ratio(row.success_count, row.request_count),
            error_rate: ratio(row.error_count, row.request_count),
            provider_id: row.provider_id,
            provider_name: row.provider_name,
            request_count: row.request_count,
            success_count: row.success_count,
            error_count: row.error_count,
            output_tokens: row.output_tokens,
            avg_output_tps: row.avg_output_tps,
            avg_ttfb_ms: row.avg_ttfb_ms,
            avg_latency_ms: row.avg_latency_ms,
            avg_response_headers_ms: row.avg_response_headers_ms,
            avg_first_output_ms: row.avg_first_output_ms,
            slow_request_count: row.slow_request_count,
        }
    }
}

impl From<RecentErrorRow> for RecentPerformanceError {
    fn from(row: RecentErrorRow) -> Self {
        Self {
            created_at: query::format_timestamp(row.created_at),
            request_id: row.request_id,
            provider_id: row.provider_id,
            provider_name: row.provider_name,
            model: row.model,
            status_code: row.status_code,
            error_type: row.error_type,
            error_message: row.error_message,
            response_headers_ms: row.response_headers_ms,
            first_output_ms: row.first_output_ms,
            latency_ms: row.latency_ms,
            ttfb_ms: row.ttfb_ms,
        }
    }
}

pub(super) fn group_error_trend(rows: Vec<ErrorTrendRow>, granularity: SnapshotGranularity) -> Vec<ErrorTrendPoint> {
    let mut groups = BTreeMap::<time::OffsetDateTime, Vec<ErrorDistributionItem>>::new();
    for row in rows {
        groups.entry(row.bucket_started_at).or_default().push(ErrorDistributionItem {
            category: row.category,
            count: row.count,
        });
    }
    groups
        .into_iter()
        .map(|(bucket_started_at, categories)| {
            let total = categories.iter().map(|item| item.count).sum();
            ErrorTrendPoint {
                bucket_started_at: query::format_timestamp(bucket_started_at),
                bucket_ended_at: query::format_timestamp(bucket_started_at + time::Duration::seconds(granularity.bucket_seconds())),
                total,
                categories,
            }
        })
        .collect()
}

fn ratio(numerator: i64, denominator: i64) -> f64 {
    if denominator <= 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64
}
