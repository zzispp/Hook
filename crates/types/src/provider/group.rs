use serde::{Deserialize, Serialize};

use crate::model::{PatchField, deserialize_patch_value};

const DEFAULT_PROVIDER_GROUP_LIMIT: u64 = 100;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
    pub provider_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderKeyGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
    pub provider_key_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ProviderGroupListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_provider_group_limit")]
    pub limit: u64,
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ProviderGroupCreate {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub provider_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ProviderKeyGroupCreate {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub provider_key_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderGroupUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub description: PatchField<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub provider_ids: PatchField<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct ProviderKeyGroupUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub description: PatchField<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub provider_key_ids: PatchField<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderGroupListResponse {
    pub groups: Vec<ProviderGroup>,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderKeyGroupListResponse {
    pub groups: Vec<ProviderKeyGroup>,
    pub total: u64,
}

impl ProviderGroupUpdate {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.description.is_missing() && self.sort_order.is_none() && self.provider_ids.is_missing()
    }
}

impl ProviderKeyGroupUpdate {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.description.is_missing() && self.sort_order.is_none() && self.provider_key_ids.is_missing()
    }
}

fn default_provider_group_limit() -> u64 {
    DEFAULT_PROVIDER_GROUP_LIMIT
}
