use std::collections::HashMap;

use storage::provider::RoutingMetricRecord;
use types::provider::{RouteIdentity, RoutingMetricSnapshot, RoutingPriorSource};

#[derive(Default)]
pub(super) struct AggregateCatalog {
    provider_model_format: HashMap<(String, String, String, bool), AggregateMetricRecord>,
    provider_model: HashMap<(String, String), AggregateMetricRecord>,
    provider: HashMap<String, AggregateMetricRecord>,
}

impl AggregateCatalog {
    pub(super) fn from_records(records: &[RoutingMetricRecord]) -> Self {
        let mut catalog = Self::default();
        for record in records {
            catalog.add(record);
        }
        catalog
    }

    pub(super) fn prior(&self, route: &RouteIdentity) -> Option<(RoutingPriorSource, &AggregateMetricRecord)> {
        self.provider_model_format
            .get(&provider_model_format_key(route))
            .map(|record| (RoutingPriorSource::ProviderModelFormat, record))
            .or_else(|| {
                self.provider_model
                    .get(&provider_model_key(route))
                    .map(|record| (RoutingPriorSource::ProviderModel, record))
            })
            .or_else(|| self.provider.get(&route.provider_id).map(|record| (RoutingPriorSource::Provider, record)))
    }

    fn add(&mut self, record: &RoutingMetricRecord) {
        self.provider_model_format
            .entry(provider_model_format_key(&record.route))
            .or_default()
            .push(record);
        self.provider_model.entry(provider_model_key(&record.route)).or_default().push(record);
        self.provider.entry(record.route.provider_id.clone()).or_default().push(record);
    }
}

#[derive(Default)]
pub(super) struct AggregateMetricRecord {
    pub(super) snapshot: RoutingMetricSnapshot,
    pub(super) last_seen_at: Option<time::OffsetDateTime>,
    latency_sum_ms: f64,
    latency_count: u64,
    first_token_sum_ms: f64,
    first_token_count: u64,
    tps_output_tokens: u64,
    tps_latency_sum_ms: f64,
}

impl AggregateMetricRecord {
    fn push(&mut self, record: &RoutingMetricRecord) {
        self.snapshot.request_count += record.snapshot.request_count;
        self.snapshot.success_count += record.snapshot.success_count;
        self.snapshot.failure_count += record.snapshot.failure_count;
        self.snapshot.timeout_count += record.snapshot.timeout_count;
        self.snapshot.rate_limited_count += record.snapshot.rate_limited_count;
        self.snapshot.server_error_count += record.snapshot.server_error_count;
        self.snapshot.format_conversion_failure_count += record.snapshot.format_conversion_failure_count;
        self.snapshot.usage_missing_count += record.snapshot.usage_missing_count;
        self.snapshot.stream_abnormal_end_count += record.snapshot.stream_abnormal_end_count;
        self.snapshot.schema_tool_call_failure_count += record.snapshot.schema_tool_call_failure_count;
        self.snapshot.total_tokens += record.snapshot.total_tokens;
        self.snapshot.sample_count += record.snapshot.sample_count;
        self.snapshot.upstream_total_cost = decimal_sum(self.snapshot.upstream_total_cost, record.snapshot.upstream_total_cost);
        self.add_latency(record);
        self.add_first_token(record);
        self.add_tps(record);
        self.snapshot.latency_avg_ms = average(self.latency_sum_ms, self.latency_count);
        self.snapshot.first_token_avg_ms = average(self.first_token_sum_ms, self.first_token_count);
        self.snapshot.output_tps = average_tps(self.tps_output_tokens, self.tps_latency_sum_ms);
        self.last_seen_at = max_time(self.last_seen_at, record.last_seen_at);
    }

    fn add_latency(&mut self, record: &RoutingMetricRecord) {
        if let Some(value) = record.snapshot.latency_avg_ms {
            self.latency_sum_ms += value * record.snapshot.sample_count as f64;
            self.latency_count += record.snapshot.sample_count;
        }
    }

    fn add_first_token(&mut self, record: &RoutingMetricRecord) {
        if let Some(value) = record.snapshot.first_token_avg_ms {
            self.first_token_sum_ms += value * record.snapshot.sample_count as f64;
            self.first_token_count += record.snapshot.sample_count;
        }
    }

    fn add_tps(&mut self, record: &RoutingMetricRecord) {
        if let Some(value) = record.snapshot.output_tps.filter(|value| *value > 0.0) {
            self.tps_output_tokens += record.snapshot.total_tokens;
            self.tps_latency_sum_ms += record.snapshot.total_tokens as f64 * 1000.0 / value;
        }
    }
}

fn provider_model_format_key(route: &RouteIdentity) -> (String, String, String, bool) {
    (
        route.provider_id.clone(),
        route.global_model_id.clone(),
        route.provider_api_format.clone(),
        route.is_stream,
    )
}

fn provider_model_key(route: &RouteIdentity) -> (String, String) {
    (route.provider_id.clone(), route.global_model_id.clone())
}

fn decimal_sum(left: Option<rust_decimal::Decimal>, right: Option<rust_decimal::Decimal>) -> Option<rust_decimal::Decimal> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn average(sum: f64, count: u64) -> Option<f64> {
    (count > 0).then(|| sum / count as f64)
}

fn average_tps(output_tokens: u64, latency_ms: f64) -> Option<f64> {
    (output_tokens > 0 && latency_ms > 0.0).then(|| output_tokens as f64 * 1000.0 / latency_ms)
}

fn max_time(left: Option<time::OffsetDateTime>, right: time::OffsetDateTime) -> Option<time::OffsetDateTime> {
    Some(left.map(|left| left.max(right)).unwrap_or(right))
}
