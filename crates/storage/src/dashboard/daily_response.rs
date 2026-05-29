use rust_decimal::Decimal;
use types::dashboard::{DashboardDailyBreakdownItem, DashboardDailyModelSummary, DashboardDailyProviderSummary, DashboardDailyStat};

use super::money::admin_cost_metrics;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct DailyMeasure {
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost: Decimal,
    pub upstream_total_cost: Decimal,
    pub latency_total_ms: f64,
    pub latency_sample_count: i64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct DailyResponseOptions {
    pub include_admin_breakdowns: bool,
    pub include_admin_costs: bool,
}

pub(super) trait DailyAggregateView {
    fn measure(&self) -> &DailyMeasure;
    fn models(&self) -> &std::collections::BTreeMap<String, DailyMeasure>;
    fn providers(&self) -> &std::collections::BTreeMap<String, DailyMeasure>;
}

pub(super) fn day_response<T>(date: time::Date, aggregate: Option<&T>, options: DailyResponseOptions) -> DashboardDailyStat
where
    T: DailyAggregateView,
{
    let empty = EmptyDailyAggregate::default();
    let value = aggregate.map_or(&empty as &dyn DailyAggregateView, |item| item as &dyn DailyAggregateView);
    let measure = value.measure();
    let metrics = admin_cost_metrics(measure.total_cost, measure.upstream_total_cost, options.include_admin_costs);
    DashboardDailyStat {
        date: date.to_string(),
        request_count: measure.request_count,
        total_tokens: measure.total_tokens,
        total_cost: measure.total_cost,
        upstream_total_cost: metrics.upstream_total_cost,
        profit: metrics.profit,
        profit_rate: metrics.profit_rate,
        avg_latency_ms: avg_latency(measure.latency_total_ms, measure.latency_sample_count),
        unique_models: value.models().len(),
        unique_providers: provider_count(value, options.include_admin_breakdowns),
        model_breakdown: breakdown_items(value.models(), options.include_admin_costs),
        provider_breakdown: provider_breakdown(value, options),
    }
}

pub(super) fn breakdown_items(values: &std::collections::BTreeMap<String, DailyMeasure>, include_admin_costs: bool) -> Vec<DashboardDailyBreakdownItem> {
    let mut items = values
        .iter()
        .map(|(name, value)| breakdown_item(name, value, include_admin_costs))
        .collect::<Vec<_>>();
    sort_by_cost_name(&mut items);
    items
}

pub(super) fn breakdown_item(name: &str, value: &DailyMeasure, include_admin_costs: bool) -> DashboardDailyBreakdownItem {
    let financial = admin_cost_metrics(value.total_cost, value.upstream_total_cost, include_admin_costs);
    DashboardDailyBreakdownItem {
        name: name.to_owned(),
        request_count: value.request_count,
        total_tokens: value.total_tokens,
        total_cost: value.total_cost,
        upstream_total_cost: financial.upstream_total_cost,
        profit: financial.profit,
        profit_rate: financial.profit_rate,
    }
}

pub(super) fn model_summary(name: String, value: DailyMeasure, include_admin_costs: bool) -> DashboardDailyModelSummary {
    let financial = admin_cost_metrics(value.total_cost, value.upstream_total_cost, include_admin_costs);
    DashboardDailyModelSummary {
        name,
        request_count: value.request_count,
        total_tokens: value.total_tokens,
        total_cost: value.total_cost,
        upstream_total_cost: financial.upstream_total_cost,
        profit: financial.profit,
        profit_rate: financial.profit_rate,
        avg_latency_ms: avg_latency(value.latency_total_ms, value.latency_sample_count),
        cost_per_request: cost_per_request(value.total_cost, value.request_count),
        tokens_per_request: tokens_per_request(value.total_tokens, value.request_count),
    }
}

pub(super) fn provider_summary(name: String, value: DailyMeasure, include_admin_costs: bool) -> DashboardDailyProviderSummary {
    let financial = admin_cost_metrics(value.total_cost, value.upstream_total_cost, include_admin_costs);
    DashboardDailyProviderSummary {
        name,
        request_count: value.request_count,
        total_tokens: value.total_tokens,
        total_cost: value.total_cost,
        upstream_total_cost: financial.upstream_total_cost,
        profit: financial.profit,
        profit_rate: financial.profit_rate,
    }
}

pub(super) fn avg_latency(total_ms: f64, samples: i64) -> Option<f64> {
    if samples <= 0 {
        return None;
    }
    Some(total_ms / samples as f64)
}

fn provider_count(value: &dyn DailyAggregateView, include_admin_breakdowns: bool) -> usize {
    if include_admin_breakdowns {
        return value.providers().len();
    }
    0
}

fn provider_breakdown(value: &dyn DailyAggregateView, options: DailyResponseOptions) -> Vec<DashboardDailyBreakdownItem> {
    if !options.include_admin_breakdowns {
        return Vec::new();
    }
    breakdown_items(value.providers(), options.include_admin_costs)
}

#[derive(Default)]
struct EmptyDailyAggregate {
    measure: DailyMeasure,
    models: std::collections::BTreeMap<String, DailyMeasure>,
    providers: std::collections::BTreeMap<String, DailyMeasure>,
}

impl DailyAggregateView for EmptyDailyAggregate {
    fn measure(&self) -> &DailyMeasure {
        &self.measure
    }

    fn models(&self) -> &std::collections::BTreeMap<String, DailyMeasure> {
        &self.models
    }

    fn providers(&self) -> &std::collections::BTreeMap<String, DailyMeasure> {
        &self.providers
    }
}

pub(super) fn sort_by_cost_name<T>(items: &mut [T])
where
    T: DailySortable,
{
    items.sort_by(|left, right| right.total_cost().cmp(&left.total_cost()).then_with(|| left.name().cmp(right.name())));
}

pub(super) trait DailySortable {
    fn name(&self) -> &str;
    fn total_cost(&self) -> Decimal;
}

impl DailySortable for DashboardDailyBreakdownItem {
    fn name(&self) -> &str {
        &self.name
    }

    fn total_cost(&self) -> Decimal {
        self.total_cost
    }
}

impl DailySortable for DashboardDailyModelSummary {
    fn name(&self) -> &str {
        &self.name
    }

    fn total_cost(&self) -> Decimal {
        self.total_cost
    }
}

impl DailySortable for DashboardDailyProviderSummary {
    fn name(&self) -> &str {
        &self.name
    }

    fn total_cost(&self) -> Decimal {
        self.total_cost
    }
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
