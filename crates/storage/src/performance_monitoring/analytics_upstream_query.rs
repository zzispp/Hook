use sea_orm::Value;
use types::performance_monitoring::PerformanceMonitoringAnalyticsRequest;

use super::{analytics_sql::plan_bucket_values, types::SnapshotQueryPlan};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct UpstreamFilters {
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub api_format: Option<String>,
    pub is_stream: Option<bool>,
    pub needs_conversion: Option<bool>,
}

impl UpstreamFilters {
    pub(super) fn from_request(request: &PerformanceMonitoringAnalyticsRequest) -> Self {
        Self {
            provider_id: normalized_string(request.provider_id.clone()),
            model: normalized_string(request.model.clone()),
            api_format: normalized_string(request.api_format.clone()),
            is_stream: request.is_stream,
            needs_conversion: request.needs_conversion,
        }
    }
}

pub(super) struct SqlParts {
    pub sql: String,
    pub values: Vec<Value>,
}

pub(super) struct QueryBuilder {
    pub sql: String,
    values: Vec<Value>,
}

impl QueryBuilder {
    pub(super) fn metric(plan: &SnapshotQueryPlan) -> Self {
        Self {
            sql: "WITH filtered AS (SELECT * FROM dashboard_request_metric_buckets b WHERE b.source_type = 'candidate' AND b.bucket_granularity = $3 AND b.bucket_started_at >= $1 AND b.bucket_started_at < $2".into(),
            values: plan_bucket_values(plan),
        }
    }

    pub(super) fn push<T>(&mut self, value: T) -> usize
    where
        Value: From<T>,
    {
        self.values.push(Value::from(value));
        self.values.len()
    }

    pub(super) fn finish(self) -> SqlParts {
        SqlParts {
            sql: self.sql,
            values: self.values,
        }
    }
}

pub(super) struct HistogramCteInput<'a> {
    pub raw_select: &'static str,
    pub raw_group: &'static str,
    pub partition: &'static str,
    pub total_select: &'static str,
    pub total_group: &'static str,
    pub provider_ids: &'a [String],
}

impl<'a> HistogramCteInput<'a> {
    pub(super) fn empty() -> Self {
        Self {
            raw_select: "",
            raw_group: "",
            partition: "",
            total_select: "",
            total_group: "",
            provider_ids: &[],
        }
    }

    pub(super) fn provider() -> Self {
        Self {
            raw_select: "COALESCE(h.provider_id, 'unknown') AS provider_id, ",
            raw_group: "COALESCE(h.provider_id, 'unknown'), ",
            partition: "provider_id, ",
            total_select: "provider_id, ",
            total_group: "provider_id, ",
            provider_ids: &[],
        }
    }

    pub(super) fn timeline(provider_ids: &'a [String]) -> Self {
        Self {
            raw_select: "h.bucket_started_at, h.bucket_ended_at, COALESCE(h.provider_id, 'unknown') AS provider_id, ",
            raw_group: "h.bucket_started_at, h.bucket_ended_at, COALESCE(h.provider_id, 'unknown'), ",
            partition: "bucket_started_at, provider_id, ",
            total_select: "bucket_started_at, bucket_ended_at, provider_id, ",
            total_group: "bucket_started_at, bucket_ended_at, provider_id, ",
            provider_ids,
        }
    }
}

pub(super) fn push_filters(query: &mut QueryBuilder, alias: &str, filters: &UpstreamFilters) {
    push_filter(query, alias, "provider_id", filters.provider_id.clone());
    push_filter(query, alias, "global_model_id", filters.model.clone());
    push_filter(query, alias, "provider_api_format", filters.api_format.clone());
    push_bool_filter(query, alias, "is_stream", filters.is_stream);
    push_bool_filter(query, alias, "needs_conversion", filters.needs_conversion);
}

pub(super) fn push_provider_id_filter(query: &mut QueryBuilder, alias: &str, provider_ids: &[String]) {
    if provider_ids.is_empty() {
        return;
    }
    let values = provider_ids
        .iter()
        .map(|provider_id| format!("${}", query.push(provider_id.clone())))
        .collect::<Vec<_>>()
        .join(", ");
    query.sql.push_str(&format!(" AND COALESCE({alias}.provider_id, 'unknown') IN ({values})"));
}

fn push_filter(query: &mut QueryBuilder, alias: &str, column: &str, value: Option<String>) {
    if let Some(value) = value {
        let index = query.push(value);
        query.sql.push_str(&format!(" AND {alias}.{column} = ${index}"));
    }
}

fn push_bool_filter(query: &mut QueryBuilder, alias: &str, column: &str, value: Option<bool>) {
    if let Some(value) = value {
        let index = query.push(value);
        query.sql.push_str(&format!(" AND {alias}.{column} = ${index}"));
    }
}

fn normalized_string(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}
