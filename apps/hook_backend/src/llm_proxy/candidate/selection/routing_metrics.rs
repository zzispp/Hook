use storage::provider::{RoutingMetricRecord, RoutingRouteStateRecord};
use types::provider::{RouteIdentity, RoutingMetricSnapshot, RoutingMetricSource, RoutingMetricWindow, RoutingPriorSource};

use crate::llm_proxy::routing::{RoutingEmaSnapshot, RoutingMetricsSnapshot};

mod aggregate;
mod context_state;

use aggregate::{AggregateCatalog, AggregateMetricRecord};
pub(super) use context_state::ContextRouteStateCatalog;

pub(super) struct MetricCatalog {
    entries: Vec<MetricCatalogEntry>,
}

pub(super) struct RouteStateCatalog {
    records: Vec<RoutingRouteStateRecord>,
}

pub(super) struct ResolvedMetric {
    pub(super) snapshot: RoutingMetricSnapshot,
    pub(super) metric_window: RoutingMetricWindow,
    pub(super) metric_freshness_seconds: i64,
    pub(super) recent_metric: Option<RoutingMetricSnapshot>,
    pub(super) metric_source: RoutingMetricSource,
    pub(super) prior_source: RoutingPriorSource,
    pub(super) prior_sample_count: u64,
}

#[derive(Clone, Copy)]
pub(super) struct RouteFingerprints<'a> {
    pub(super) route_config: &'a str,
    pub(super) price_config: &'a str,
}

impl MetricCatalog {
    pub(super) fn from_snapshot(snapshot: &RoutingMetricsSnapshot, windows: Vec<RoutingMetricWindow>) -> Self {
        let entries = windows
            .into_iter()
            .map(|window| {
                let records = snapshot.windows.get(&window).cloned().unwrap_or_default();
                MetricCatalogEntry::new(window, records)
            })
            .collect();
        Self { entries }
    }

    pub(super) fn resolve(
        &self,
        route: &RouteIdentity,
        fingerprints: RouteFingerprints<'_>,
        min_samples: u64,
        requested_window: RoutingMetricWindow,
    ) -> ResolvedMetric {
        if let Some((window, record)) = self
            .best_record(route, fingerprints, min_samples)
            .or_else(|| self.richest_record(route, fingerprints))
        {
            return self.resolved_exact(route, fingerprints, requested_window, window, record);
        }
        if let Some((window, record, source)) = self.best_prior(route, min_samples).or_else(|| self.richest_prior(route)) {
            return resolved_prior(window, record, source);
        }
        resolved_neutral(requested_window)
    }

    fn resolved_exact(
        &self,
        route: &RouteIdentity,
        fingerprints: RouteFingerprints<'_>,
        requested_window: RoutingMetricWindow,
        window: RoutingMetricWindow,
        record: &RoutingMetricRecord,
    ) -> ResolvedMetric {
        let recent_metric = (window != requested_window)
            .then(|| self.record(route, fingerprints, requested_window).map(|(_, record)| record.snapshot.clone()))
            .flatten();
        ResolvedMetric {
            snapshot: record.snapshot.clone(),
            metric_window: window,
            metric_freshness_seconds: freshness_seconds(record.last_seen_at),
            recent_metric,
            metric_source: if window == requested_window {
                RoutingMetricSource::Exact
            } else {
                RoutingMetricSource::WindowFallback
            },
            prior_source: RoutingPriorSource::ExactRoute,
            prior_sample_count: record.snapshot.sample_count,
        }
    }

    fn record(
        &self,
        route: &RouteIdentity,
        fingerprints: RouteFingerprints<'_>,
        window: RoutingMetricWindow,
    ) -> Option<(RoutingMetricWindow, &RoutingMetricRecord)> {
        self.entries.iter().find(|entry| entry.window == window).and_then(|entry| {
            entry
                .records
                .iter()
                .find(|record| record.route == *route && fingerprint_matches(record, fingerprints))
                .map(|record| (entry.window, record))
        })
    }

    fn best_record(&self, route: &RouteIdentity, fingerprints: RouteFingerprints<'_>, min_samples: u64) -> Option<(RoutingMetricWindow, &RoutingMetricRecord)> {
        self.entries.iter().find_map(|entry| {
            entry
                .records
                .iter()
                .find(|record| record.route == *route)
                .filter(|record| fingerprint_matches(record, fingerprints))
                .filter(|record| record.snapshot.sample_count >= min_samples)
                .map(|record| (entry.window, record))
        })
    }

    fn richest_record(&self, route: &RouteIdentity, fingerprints: RouteFingerprints<'_>) -> Option<(RoutingMetricWindow, &RoutingMetricRecord)> {
        self.entries
            .iter()
            .filter_map(|entry| {
                entry
                    .records
                    .iter()
                    .filter(|record| record.route == *route)
                    .find(|record| fingerprint_matches(record, fingerprints))
                    .map(|record| (entry.window, record))
            })
            .max_by_key(|(_, record)| record.snapshot.sample_count)
    }

    fn best_prior(&self, route: &RouteIdentity, min_samples: u64) -> Option<(RoutingMetricWindow, &AggregateMetricRecord, RoutingPriorSource)> {
        self.entries.iter().find_map(|entry| entry.best_prior(route, min_samples))
    }

    fn richest_prior(&self, route: &RouteIdentity) -> Option<(RoutingMetricWindow, &AggregateMetricRecord, RoutingPriorSource)> {
        self.entries
            .iter()
            .filter_map(|entry| entry.richest_prior(route))
            .max_by_key(|(_, record, _)| record.snapshot.sample_count)
    }
}

impl RouteStateCatalog {
    pub(super) fn from_snapshot(snapshot: &RoutingMetricsSnapshot) -> Self {
        Self {
            records: snapshot.route_states.clone(),
        }
    }

    pub(super) fn record(&self, route: &RouteIdentity, fingerprints: RouteFingerprints<'_>) -> Option<RoutingEmaSnapshot> {
        let record = self
            .records
            .iter()
            .find(|record| record.route == *route && route_state_fingerprint_matches(record, fingerprints))?;
        Some(RoutingEmaSnapshot {
            success_rate: record.ema_success_rate,
            ttfb_avg_ms: record.ema_ttfb_ms,
            latency_avg_ms: record.ema_latency_ms,
            output_tps: record.ema_output_tps,
            sample_count: record.sample_count,
            freshness_seconds: freshness_seconds(record.last_updated_at),
        })
    }
}

struct MetricCatalogEntry {
    window: RoutingMetricWindow,
    records: Vec<RoutingMetricRecord>,
    aggregates: AggregateCatalog,
}

impl MetricCatalogEntry {
    fn new(window: RoutingMetricWindow, records: Vec<RoutingMetricRecord>) -> Self {
        let aggregates = AggregateCatalog::from_records(&records);
        Self { window, records, aggregates }
    }

    fn best_prior(&self, route: &RouteIdentity, min_samples: u64) -> Option<(RoutingMetricWindow, &AggregateMetricRecord, RoutingPriorSource)> {
        self.aggregates
            .prior(route)
            .filter(|(_, record)| record.snapshot.sample_count >= min_samples)
            .map(|(source, record)| (self.window, record, source))
    }

    fn richest_prior(&self, route: &RouteIdentity) -> Option<(RoutingMetricWindow, &AggregateMetricRecord, RoutingPriorSource)> {
        self.aggregates.prior(route).map(|(source, record)| (self.window, record, source))
    }
}

fn resolved_prior(window: RoutingMetricWindow, record: &AggregateMetricRecord, source: RoutingPriorSource) -> ResolvedMetric {
    ResolvedMetric {
        snapshot: record.snapshot.clone(),
        metric_window: window,
        metric_freshness_seconds: record.last_seen_at.map(freshness_seconds).unwrap_or(0),
        recent_metric: None,
        metric_source: RoutingMetricSource::Prior,
        prior_source: source,
        prior_sample_count: record.snapshot.sample_count,
    }
}

fn resolved_neutral(window: RoutingMetricWindow) -> ResolvedMetric {
    ResolvedMetric {
        snapshot: RoutingMetricSnapshot::default(),
        metric_window: window,
        metric_freshness_seconds: 0,
        recent_metric: None,
        metric_source: RoutingMetricSource::Prior,
        prior_source: RoutingPriorSource::Neutral,
        prior_sample_count: 0,
    }
}

fn freshness_seconds(value: time::OffsetDateTime) -> i64 {
    (time::OffsetDateTime::now_utc() - value).whole_seconds().max(0)
}

fn fingerprint_matches(record: &RoutingMetricRecord, fingerprints: RouteFingerprints<'_>) -> bool {
    record.route_config_fingerprint.as_deref() == Some(fingerprints.route_config)
        && record.price_config_fingerprint.as_deref() == Some(fingerprints.price_config)
}

fn route_state_fingerprint_matches(record: &RoutingRouteStateRecord, fingerprints: RouteFingerprints<'_>) -> bool {
    record.route_config_fingerprint.as_deref() == Some(fingerprints.route_config)
        && record.price_config_fingerprint.as_deref() == Some(fingerprints.price_config)
}

#[cfg(test)]
mod tests;
