use storage::provider::RoutingContextRouteStateRecord;
use types::provider::{ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1, RouteIdentity, RoutingProfileId};

use crate::llm_proxy::routing::RoutingMetricsSnapshot;

use super::RouteFingerprints;

pub(crate) struct ContextRouteStateCatalog {
    records: Vec<RoutingContextRouteStateRecord>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ContextRouteSamples {
    pub(crate) route_sample_count: u64,
    pub(crate) total_sample_count: u64,
}

impl ContextRouteStateCatalog {
    pub(crate) fn from_snapshot(snapshot: &RoutingMetricsSnapshot) -> Self {
        let records = snapshot
            .context_route_states
            .iter()
            .filter(|record| record.timing_metric_semantics_version == ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1)
            .cloned()
            .collect();
        Self { records }
    }

    pub(crate) fn samples(
        &self,
        profile_id: RoutingProfileId,
        context_key: &str,
        route: &RouteIdentity,
        fingerprints: RouteFingerprints<'_>,
    ) -> ContextRouteSamples {
        let context_records = self
            .records
            .iter()
            .filter(|record| record.profile_id == profile_id.as_str() && record.context_key == context_key);
        let mut samples = ContextRouteSamples::default();
        for record in context_records {
            samples.total_sample_count += record.sample_count;
            if record.route == *route && fingerprint_matches(record, fingerprints) {
                samples.route_sample_count += record.sample_count;
            }
        }
        ContextRouteSamples {
            route_sample_count: samples.route_sample_count,
            total_sample_count: samples.total_sample_count,
        }
    }
}

fn fingerprint_matches(record: &RoutingContextRouteStateRecord, fingerprints: RouteFingerprints<'_>) -> bool {
    record.route_config_fingerprint.as_deref() == Some(fingerprints.route_config)
        && record.price_config_fingerprint.as_deref() == Some(fingerprints.price_config)
}
