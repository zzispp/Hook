use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use types::provider::{
    RouteIdentity, RouteScoreExplanation, RoutingDecisionResponse, RoutingMetricSnapshot, RoutingMetricWindow, RoutingProfile, RoutingProfileWeights,
};

use crate::StorageResult;

use super::repository::ProviderStore;

#[derive(Clone, Debug)]
pub struct RoutingMetricRecord {
    pub route: RouteIdentity,
    pub provider_name: Option<String>,
    pub key_name: Option<String>,
    pub endpoint_name: Option<String>,
    pub snapshot: RoutingMetricSnapshot,
    pub last_seen_at: time::OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct RoutingProfileVersionSnapshot {
    pub profile_id: String,
    pub profile_version: String,
    pub admin_weights: RoutingProfileWeights,
    pub learned_weights: Option<RoutingProfileWeights>,
    pub effective_weights: RoutingProfileWeights,
    pub reward_window: RoutingMetricWindow,
    pub sample_count: u64,
    pub created_at: time::OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct RoutingMetricDelta {
    pub route: RouteIdentity,
    pub provider_name: Option<String>,
    pub key_name: Option<String>,
    pub endpoint_name: Option<String>,
    pub request_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub timeout_count: i64,
    pub rate_limited_count: i64,
    pub server_error_count: i64,
    pub latency_sum_ms: i64,
    pub latency_sample_count: i64,
    pub ttfb_sum_ms: i64,
    pub ttfb_sample_count: i64,
    pub output_tokens: i64,
    pub tps_latency_sum_ms: i64,
    pub tps_sample_count: i64,
    pub upstream_total_cost: Decimal,
    pub total_tokens: i64,
    pub observed_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct DecisionSamplePayload {
    pub(super) candidates: Vec<RouteScoreExplanation>,
}

impl ProviderStore {
    pub async fn upsert_routing_metric_delta(&self, delta: RoutingMetricDelta) -> StorageResult<()> {
        super::routing_metric_repository::upsert_metric_delta(self.connection(), delta).await
    }

    pub async fn list_routing_metrics(&self, window: RoutingMetricWindow) -> StorageResult<Vec<RoutingMetricRecord>> {
        super::routing_metric_repository::list_metrics(self.connection(), window).await
    }

    pub async fn upsert_routing_decision_sample(
        &self,
        request_id: &str,
        profile_id: &str,
        profile_version: &str,
        selected: Option<&RouteIdentity>,
        candidates: &[RouteScoreExplanation],
    ) -> StorageResult<()> {
        super::routing_decision_repository::upsert_decision_sample(self.connection(), request_id, profile_id, profile_version, selected, candidates).await
    }

    pub async fn get_routing_decision_sample(&self, request_id: &str) -> StorageResult<Option<RoutingDecisionResponse>> {
        super::routing_decision_repository::get_decision_sample(self.connection(), request_id).await
    }

    pub async fn list_routing_profile_overlays(&self) -> StorageResult<Vec<RoutingProfile>> {
        super::routing_profile_repository::list_profiles(self.connection()).await
    }

    pub async fn upsert_routing_profile_overlay(&self, profile: RoutingProfile) -> StorageResult<RoutingProfile> {
        super::routing_profile_repository::upsert_profile(self.connection(), profile).await
    }

    pub async fn get_latest_routing_profile_version(&self, profile_id: &str) -> StorageResult<Option<RoutingProfileVersionSnapshot>> {
        super::routing_profile_version_repository::latest_profile_version(self.connection(), profile_id).await
    }

    pub async fn insert_routing_profile_version_snapshot(&self, snapshot: &RoutingProfileVersionSnapshot) -> StorageResult<()> {
        super::routing_profile_version_repository::insert_profile_version(self.connection(), snapshot).await
    }
}
