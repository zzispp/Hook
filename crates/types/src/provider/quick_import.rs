use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::quick_import_sync::ProviderQuickImportSyncConfig;
use super::{Provider, ProviderApiKey, ProviderEndpoint, ProviderModelBinding, ProviderModelCost};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuickImportSourceKind {
    Newapi,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderQuickImportSourceConfig {
    Newapi(NewApiQuickImportConfig),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewApiQuickImportConfig {
    pub base_url: String,
    pub system_access_token: String,
    pub user_id: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderQuickImportPreviewRequest {
    pub source_kind: ProviderQuickImportSourceKind,
    pub source: ProviderQuickImportSourceConfig,
    pub provider_name: String,
    #[serde(default)]
    pub provider_config: ProviderQuickImportProviderConfig,
    #[serde(default = "default_recharge_multiplier", with = "rust_decimal::serde::float")]
    pub recharge_multiplier: Decimal,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderQuickImportCommitRequest {
    pub source_kind: ProviderQuickImportSourceKind,
    pub source: ProviderQuickImportSourceConfig,
    pub provider_name: String,
    #[serde(default)]
    pub provider_config: ProviderQuickImportProviderConfig,
    #[serde(default = "default_recharge_multiplier", with = "rust_decimal::serde::float")]
    pub recharge_multiplier: Decimal,
    #[serde(default)]
    pub selected_tokens: Vec<ProviderQuickImportSelectedToken>,
    #[serde(default)]
    pub selected_model_ids: Vec<String>,
    #[serde(default)]
    pub model_mappings: Vec<ProviderQuickImportModelMappingInput>,
    #[serde(default)]
    pub sync_config: ProviderQuickImportSyncConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderQuickImportAppendPreviewRequest {
    #[serde(default)]
    pub include_linked_tokens: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderQuickImportAppendCommitRequest {
    #[serde(default)]
    pub selected_tokens: Vec<ProviderQuickImportSelectedToken>,
    #[serde(default)]
    pub selected_model_ids: Vec<String>,
    #[serde(default)]
    pub model_mappings: Vec<ProviderQuickImportModelMappingInput>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderQuickImportRelinkRequest {
    pub upstream_token_id: String,
    #[serde(default)]
    pub selected_model_ids: Vec<String>,
    #[serde(default)]
    pub model_mappings: Vec<ProviderQuickImportModelMappingInput>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderQuickImportSelectedToken {
    pub upstream_token_id: String,
    pub name: String,
    pub endpoint_formats: Vec<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub effective_cost_multiplier: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderQuickImportModelMappingInput {
    pub upstream_model_id: String,
    pub global_model_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderQuickImportProviderConfig {
    #[serde(default)]
    pub max_retries: Option<i32>,
    #[serde(default)]
    pub request_timeout_seconds: Option<f64>,
    #[serde(default)]
    pub stream_first_byte_timeout_seconds: Option<f64>,
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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportPreviewResponse {
    pub provider_id: Option<String>,
    pub source_kind: ProviderQuickImportSourceKind,
    pub provider_name: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_multiplier: Decimal,
    pub tokens: Vec<ProviderQuickImportTokenPreview>,
    pub model_mappings: Vec<ProviderQuickImportModelMappingPreview>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportTokenPreview {
    pub upstream_token_id: String,
    pub name: String,
    pub masked_key: String,
    pub status: i32,
    pub group: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub group_ratio: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub effective_cost_multiplier: Decimal,
    pub importable: bool,
    pub already_imported: bool,
    pub import_block_reason: Option<String>,
    pub linked_key: Option<ProviderQuickImportLinkedKeyPreview>,
    pub models: Vec<ProviderQuickImportRemoteModel>,
    pub cost_issues: Vec<ProviderQuickImportCostIssue>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportLinkedKeyPreview {
    pub key_id: String,
    pub name: String,
    pub endpoint_formats: Vec<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub effective_cost_multiplier: Decimal,
    pub model_mappings: Vec<ProviderQuickImportModelMappingInput>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportRemoteModel {
    pub upstream_model_id: String,
    pub suggested_global_model_id: Option<String>,
    pub supported_endpoint_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportModelMappingPreview {
    pub upstream_model_id: String,
    pub suggested_global_model_id: Option<String>,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportCostIssue {
    pub upstream_model_id: String,
    pub global_model_id: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportCommitResponse {
    pub provider: Provider,
    pub endpoints: Vec<ProviderEndpoint>,
    pub api_keys: Vec<ProviderApiKey>,
    pub model_bindings: Vec<ProviderModelBinding>,
    pub model_costs: Vec<ProviderModelCost>,
    pub imported_token_count: usize,
    pub imported_model_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportResolutionResponse {
    pub provider_id: String,
    pub key_id: String,
    pub key_name: String,
    pub source_kind: ProviderQuickImportSourceKind,
    pub current_upstream_token_id: String,
    pub current_upstream_group: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub current_effective_cost_multiplier: Decimal,
    pub statuses: Vec<super::quick_import_sync::ProviderQuickImportSyncStatus>,
    pub tokens: Vec<ProviderQuickImportTokenPreview>,
    pub model_mappings: Vec<ProviderQuickImportModelMappingPreview>,
    pub associated_models: Vec<super::ProviderKeyModelMapping>,
}

impl ProviderQuickImportSourceConfig {
    pub fn kind(&self) -> ProviderQuickImportSourceKind {
        match self {
            Self::Newapi(_) => ProviderQuickImportSourceKind::Newapi,
        }
    }
}

impl ProviderQuickImportSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Newapi => "newapi",
        }
    }
}

impl TryFrom<&str> for ProviderQuickImportSourceKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "newapi" => Ok(Self::Newapi),
            other => Err(format!("invalid quick import source kind: {other}")),
        }
    }
}

fn default_recharge_multiplier() -> Decimal {
    Decimal::ONE
}
