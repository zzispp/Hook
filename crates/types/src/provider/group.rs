use serde::{Deserialize, Serialize};

use crate::model::{PatchField, deserialize_patch_value};

const DEFAULT_PROVIDER_KEY_GROUP_LIMIT: u64 = 100;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderKeyGroupMember {
    pub provider_key_id: String,
    pub priority: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ProviderKeyGroupMemberInput {
    pub provider_key_id: String,
    pub priority: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderKeyGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
    pub provider_key_members: Vec<ProviderKeyGroupMember>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ProviderKeyGroupListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_provider_key_group_limit")]
    pub limit: u64,
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ProviderKeyGroupCreate {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub provider_key_members: Vec<ProviderKeyGroupMemberInput>,
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
    pub provider_key_members: PatchField<Vec<ProviderKeyGroupMemberInput>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderKeyGroupListResponse {
    pub groups: Vec<ProviderKeyGroup>,
    pub total: u64,
}

impl ProviderKeyGroupUpdate {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.description.is_missing() && self.sort_order.is_none() && self.provider_key_members.is_missing()
    }
}

fn default_provider_key_group_limit() -> u64 {
    DEFAULT_PROVIDER_KEY_GROUP_LIMIT
}
