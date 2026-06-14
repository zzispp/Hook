use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::quick_import::{
    ProviderQuickImportModelMappingInput, ProviderQuickImportPreviewResponse, ProviderQuickImportSourceConfig, ProviderQuickImportSourceKind,
};
use super::quick_import_sync::ProviderQuickImportSyncConfig;
use super::{Provider, ProviderApiKey, ProviderEndpoint, ProviderModelBinding, ProviderModelCost};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderQuickImportBindPreviewRequest {
    pub source_kind: ProviderQuickImportSourceKind,
    pub source: ProviderQuickImportSourceConfig,
    #[serde(default = "default_recharge_multiplier", with = "rust_decimal::serde::float")]
    pub recharge_multiplier: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportBindPreviewResponse {
    pub provider: Provider,
    pub local_keys: Vec<ProviderQuickImportBindLocalKey>,
    pub preview: ProviderQuickImportPreviewResponse,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportBindLocalKey {
    pub id: String,
    pub name: String,
    pub api_formats: Vec<String>,
    pub allowed_model_ids: Vec<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderQuickImportBindSelectedToken {
    pub upstream_token_id: String,
    #[serde(default)]
    pub local_key_id: Option<String>,
    pub name: String,
    pub endpoint_formats: Vec<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub effective_cost_multiplier: Decimal,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ProviderQuickImportBindCommitRequest {
    pub source_kind: ProviderQuickImportSourceKind,
    pub source: ProviderQuickImportSourceConfig,
    #[serde(default = "default_recharge_multiplier", with = "rust_decimal::serde::float")]
    pub recharge_multiplier: Decimal,
    #[serde(default)]
    pub selected_tokens: Vec<ProviderQuickImportBindSelectedToken>,
    #[serde(default)]
    pub selected_model_ids: Vec<String>,
    #[serde(default)]
    pub model_mappings: Vec<ProviderQuickImportModelMappingInput>,
    #[serde(default)]
    pub sync_config: ProviderQuickImportSyncConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProviderQuickImportBindCommitResponse {
    pub provider: Provider,
    pub endpoints: Vec<ProviderEndpoint>,
    pub api_keys: Vec<ProviderApiKey>,
    pub model_bindings: Vec<ProviderModelBinding>,
    pub model_costs: Vec<ProviderModelCost>,
    pub bound_token_count: usize,
    pub created_key_count: usize,
    pub reused_key_count: usize,
    pub deleted_key_count: usize,
}

fn default_recharge_multiplier() -> Decimal {
    Decimal::ONE
}
