use serde::{Deserialize, Serialize};

use super::quick_import_sync::ProviderQuickImportSyncStatus;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderQuickImportUpstreamModelSnapshot {
    pub upstream_model_id: String,
    #[serde(default)]
    pub supported_endpoint_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderQuickImportSyncEventPayload {
    pub provider_name: String,
    pub local_key_name: Option<String>,
    pub upstream_token_name: Option<String>,
    pub upstream_token_id: Option<String>,
    pub status: ProviderQuickImportSyncStatus,
    pub anomaly_summary: String,
    pub action_summary: String,
    #[serde(default)]
    pub missing_upstream_model_ids: Vec<String>,
    #[serde(default)]
    pub upstream_models_snapshot: Vec<ProviderQuickImportUpstreamModelSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportSyncEventSnapshotStatus {
    Available,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderQuickImportSyncEventDetailResponse {
    pub id: String,
    pub status: ProviderQuickImportSyncStatus,
    pub title: String,
    pub detail: String,
    pub created_at: String,
    pub snapshot_status: ProviderQuickImportSyncEventSnapshotStatus,
    pub payload: Option<ProviderQuickImportSyncEventPayload>,
}
