use serde::{Deserialize, Serialize};

use crate::model::{PatchField, deserialize_patch_value};

use super::quick_import_sync::ProviderQuickImportSyncStatus;

const DEFAULT_PROVIDER_LIMIT: u64 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOrigin {
    Manual,
    QuickImport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportAuthMode {
    Password,
    Token,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderQuickImportSourceSummary {
    pub source_kind: String,
    pub auth_mode: Option<ProviderQuickImportAuthMode>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub provider_origin: ProviderOrigin,
    pub quick_import_source: Option<ProviderQuickImportSourceSummary>,
    pub max_retries: Option<i32>,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    pub stream_first_output_timeout_seconds: Option<f64>,
    pub stream_idle_timeout_seconds: Option<f64>,
    pub priority: i32,
    pub keep_priority_on_conversion: bool,
    pub enable_format_conversion: bool,
    pub is_active: bool,
    pub quick_import_sync_summary: Option<ProviderQuickImportSyncSummary>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportSyncIssueScope {
    Source,
    Key,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportSyncIssueSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportSyncIssue {
    pub scope: ProviderQuickImportSyncIssueScope,
    pub status: ProviderQuickImportSyncStatus,
    pub severity: ProviderQuickImportSyncIssueSeverity,
    pub key_id: Option<String>,
    pub key_name: Option<String>,
    pub message: Option<String>,
    pub last_synced_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportSyncSummary {
    pub severity: ProviderQuickImportSyncIssueSeverity,
    pub issue_count: u32,
    pub affected_key_count: u32,
    pub last_synced_at: Option<String>,
    pub issues: Vec<ProviderQuickImportSyncIssue>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ProviderListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_provider_limit")]
    pub limit: u64,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub api_format: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderCreate {
    pub name: String,
    pub provider_type: String,
    #[serde(default)]
    pub max_retries: Option<i32>,
    #[serde(default)]
    pub request_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub stream_first_byte_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub stream_first_output_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub stream_idle_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub keep_priority_on_conversion: Option<bool>,
    #[serde(default)]
    pub enable_format_conversion: Option<bool>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub provider_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub max_retries: PatchField<i32>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub request_timeout_seconds: PatchField<f64>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub stream_first_byte_timeout_seconds: PatchField<f64>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub stream_first_output_timeout_seconds: PatchField<f64>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub stream_idle_timeout_seconds: PatchField<f64>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub keep_priority_on_conversion: Option<bool>,
    #[serde(default)]
    pub enable_format_conversion: Option<bool>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderListResponse {
    pub providers: Vec<Provider>,
    pub total: u64,
}

impl ProviderUpdate {
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.provider_type.is_none()
            && self.max_retries.is_missing()
            && self.request_timeout_seconds.is_missing()
            && self.stream_first_byte_timeout_seconds.is_missing()
            && self.stream_first_output_timeout_seconds.is_missing()
            && self.stream_idle_timeout_seconds.is_missing()
            && self.priority.is_none()
            && self.keep_priority_on_conversion.is_none()
            && self.enable_format_conversion.is_none()
            && self.is_active.is_none()
    }
}

impl ProviderOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::QuickImport => "quick_import",
        }
    }
}

impl TryFrom<&str> for ProviderOrigin {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "manual" => Ok(Self::Manual),
            "quick_import" => Ok(Self::QuickImport),
            other => Err(format!("invalid provider_origin: {other}")),
        }
    }
}

fn default_provider_limit() -> u64 {
    DEFAULT_PROVIDER_LIMIT
}
