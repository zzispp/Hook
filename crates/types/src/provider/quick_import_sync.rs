use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::quick_import::ProviderQuickImportSourceKind;

const DEFAULT_FETCH_FAILURE_DISABLE_THRESHOLD: u32 = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportCostSyncMode {
    #[default]
    Overwrite,
    ReportOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportUpstreamAnomalyAction {
    #[default]
    DisableKey,
    ReportOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportGroupChangedAction {
    #[default]
    DisableKey,
    ReportOnly,
    Sync,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderQuickImportAnomalyActions {
    #[serde(default)]
    pub token_deleted: ProviderQuickImportUpstreamAnomalyAction,
    #[serde(default)]
    pub token_disabled: ProviderQuickImportUpstreamAnomalyAction,
    #[serde(default)]
    pub group_removed: ProviderQuickImportUpstreamAnomalyAction,
    #[serde(default)]
    pub group_changed: ProviderQuickImportGroupChangedAction,
    #[serde(default)]
    pub key_unavailable: ProviderQuickImportUpstreamAnomalyAction,
    #[serde(default)]
    pub model_removed: ProviderQuickImportUpstreamAnomalyAction,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportFetchFailureAction {
    #[default]
    ReportOnly,
    DisableAfterFailures,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportSyncStatus {
    Ok,
    SyncDisabled,
    SourceNotConfigured,
    SourceFetchFailed,
    UpstreamTokenDeleted,
    UpstreamTokenDisabled,
    UpstreamGroupRemoved,
    UpstreamGroupChanged,
    UpstreamKeyUnavailable,
    UpstreamModelRemoved,
    NoAssociatedModels,
    CostUnavailable,
    CostPendingUpdate,
    ModelCandidateAvailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderQuickImportSyncConfig {
    #[serde(default = "default_auto_sync_enabled")]
    pub auto_sync_enabled: bool,
    #[serde(default)]
    pub cost_sync_mode: ProviderQuickImportCostSyncMode,
    #[serde(default)]
    pub anomaly_actions: ProviderQuickImportAnomalyActions,
    #[serde(default)]
    pub fetch_failure_action: ProviderQuickImportFetchFailureAction,
    #[serde(default = "default_fetch_failure_disable_threshold")]
    pub fetch_failure_disable_threshold: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportKeySyncInfo {
    pub source_kind: ProviderQuickImportSourceKind,
    pub upstream_token_id: String,
    pub upstream_group: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub upstream_group_ratio: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub effective_cost_multiplier: Decimal,
    pub statuses: Vec<ProviderQuickImportSyncStatus>,
    pub last_synced_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportSyncSettingsResponse {
    pub provider_id: String,
    pub source_kind: Option<ProviderQuickImportSourceKind>,
    pub base_url: Option<String>,
    pub user_id: Option<String>,
    pub has_system_access_token: bool,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub recharge_multiplier: Option<Decimal>,
    pub sync_config: ProviderQuickImportSyncConfig,
    pub last_status: Option<ProviderQuickImportSyncStatus>,
    pub last_error: Option<String>,
    pub last_synced_at: Option<String>,
    pub consecutive_failures: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderQuickImportSyncSettingsUpdate {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub system_access_token: Option<String>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub recharge_multiplier: Option<Decimal>,
    #[serde(default)]
    pub sync_config: Option<ProviderQuickImportSyncConfig>,
}

impl ProviderQuickImportCostSyncMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overwrite => "overwrite",
            Self::ReportOnly => "report_only",
        }
    }
}

impl TryFrom<&str> for ProviderQuickImportCostSyncMode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "overwrite" => Ok(Self::Overwrite),
            "report_only" => Ok(Self::ReportOnly),
            other => Err(format!("invalid quick import cost sync mode: {other}")),
        }
    }
}

impl ProviderQuickImportUpstreamAnomalyAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DisableKey => "disable_key",
            Self::ReportOnly => "report_only",
        }
    }
}

impl TryFrom<&str> for ProviderQuickImportUpstreamAnomalyAction {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "disable_key" => Ok(Self::DisableKey),
            "report_only" => Ok(Self::ReportOnly),
            other => Err(format!("invalid quick import anomaly action: {other}")),
        }
    }
}

impl ProviderQuickImportGroupChangedAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DisableKey => "disable_key",
            Self::ReportOnly => "report_only",
            Self::Sync => "sync",
        }
    }
}

impl TryFrom<&str> for ProviderQuickImportGroupChangedAction {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "disable_key" => Ok(Self::DisableKey),
            "report_only" => Ok(Self::ReportOnly),
            "sync" => Ok(Self::Sync),
            other => Err(format!("invalid quick import group changed action: {other}")),
        }
    }
}

impl ProviderQuickImportFetchFailureAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReportOnly => "report_only",
            Self::DisableAfterFailures => "disable_after_failures",
        }
    }
}

impl TryFrom<&str> for ProviderQuickImportFetchFailureAction {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "report_only" => Ok(Self::ReportOnly),
            "disable_after_failures" => Ok(Self::DisableAfterFailures),
            other => Err(format!("invalid quick import fetch failure action: {other}")),
        }
    }
}

impl ProviderQuickImportSyncStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::SyncDisabled => "sync_disabled",
            Self::SourceNotConfigured => "source_not_configured",
            Self::SourceFetchFailed => "source_fetch_failed",
            Self::UpstreamTokenDeleted => "upstream_token_deleted",
            Self::UpstreamTokenDisabled => "upstream_token_disabled",
            Self::UpstreamGroupRemoved => "upstream_group_removed",
            Self::UpstreamGroupChanged => "upstream_group_changed",
            Self::UpstreamKeyUnavailable => "upstream_key_unavailable",
            Self::UpstreamModelRemoved => "upstream_model_removed",
            Self::NoAssociatedModels => "no_associated_models",
            Self::CostUnavailable => "cost_unavailable",
            Self::CostPendingUpdate => "cost_pending_update",
            Self::ModelCandidateAvailable => "model_candidate_available",
        }
    }
}

impl TryFrom<&str> for ProviderQuickImportSyncStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ok" => Ok(Self::Ok),
            "sync_disabled" => Ok(Self::SyncDisabled),
            "source_not_configured" => Ok(Self::SourceNotConfigured),
            "source_fetch_failed" => Ok(Self::SourceFetchFailed),
            "upstream_token_deleted" => Ok(Self::UpstreamTokenDeleted),
            "upstream_token_disabled" => Ok(Self::UpstreamTokenDisabled),
            "upstream_group_removed" => Ok(Self::UpstreamGroupRemoved),
            "upstream_group_changed" => Ok(Self::UpstreamGroupChanged),
            "upstream_key_unavailable" => Ok(Self::UpstreamKeyUnavailable),
            "upstream_model_removed" => Ok(Self::UpstreamModelRemoved),
            "no_associated_models" => Ok(Self::NoAssociatedModels),
            "cost_unavailable" => Ok(Self::CostUnavailable),
            "cost_pending_update" => Ok(Self::CostPendingUpdate),
            "model_candidate_available" => Ok(Self::ModelCandidateAvailable),
            other => Err(format!("invalid quick import sync status: {other}")),
        }
    }
}

impl Default for ProviderQuickImportSyncConfig {
    fn default() -> Self {
        Self {
            auto_sync_enabled: true,
            cost_sync_mode: ProviderQuickImportCostSyncMode::Overwrite,
            anomaly_actions: ProviderQuickImportAnomalyActions::default(),
            fetch_failure_action: ProviderQuickImportFetchFailureAction::ReportOnly,
            fetch_failure_disable_threshold: DEFAULT_FETCH_FAILURE_DISABLE_THRESHOLD,
        }
    }
}

impl Default for ProviderQuickImportAnomalyActions {
    fn default() -> Self {
        Self {
            token_deleted: ProviderQuickImportUpstreamAnomalyAction::DisableKey,
            token_disabled: ProviderQuickImportUpstreamAnomalyAction::DisableKey,
            group_removed: ProviderQuickImportUpstreamAnomalyAction::DisableKey,
            group_changed: ProviderQuickImportGroupChangedAction::DisableKey,
            key_unavailable: ProviderQuickImportUpstreamAnomalyAction::ReportOnly,
            model_removed: ProviderQuickImportUpstreamAnomalyAction::DisableKey,
        }
    }
}

fn default_auto_sync_enabled() -> bool {
    true
}

fn default_fetch_failure_disable_threshold() -> u32 {
    DEFAULT_FETCH_FAILURE_DISABLE_THRESHOLD
}
