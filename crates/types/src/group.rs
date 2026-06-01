use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::{PatchField, deserialize_patch_value};

const DEFAULT_GROUP_LIMIT: u64 = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillingGroup {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub billing_multiplier: Decimal,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub allowed_provider_key_ids: Vec<String>,
    pub visible_user_group_codes: Vec<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct BillingGroupListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_group_limit")]
    pub limit: u64,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BillingGroupCreate {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub billing_multiplier: Decimal,
    #[serde(default)]
    pub allowed_model_ids: Vec<String>,
    #[serde(default)]
    pub allowed_provider_ids: Vec<String>,
    #[serde(default)]
    pub allowed_provider_key_ids: Vec<String>,
    #[serde(default)]
    pub visible_user_group_codes: Vec<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct BillingGroupUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub description: PatchField<String>,
    #[serde(default)]
    pub billing_multiplier: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub allowed_model_ids: PatchField<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub allowed_provider_ids: PatchField<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub allowed_provider_key_ids: PatchField<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_patch_value")]
    pub visible_user_group_codes: PatchField<Vec<String>>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BillingGroupResponse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub billing_multiplier: Decimal,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub allowed_provider_key_ids: Vec<String>,
    pub visible_user_group_codes: Vec<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BillingGroupListResponse {
    pub groups: Vec<BillingGroupResponse>,
    pub total: u64,
}

impl BillingGroupUpdate {
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_missing()
            && self.billing_multiplier.is_none()
            && self.allowed_model_ids.is_missing()
            && self.allowed_provider_ids.is_missing()
            && self.allowed_provider_key_ids.is_missing()
            && self.visible_user_group_codes.is_missing()
            && self.is_active.is_none()
            && self.sort_order.is_none()
    }
}

impl From<BillingGroup> for BillingGroupResponse {
    fn from(value: BillingGroup) -> Self {
        Self {
            id: value.id,
            code: value.code,
            name: value.name,
            description: value.description,
            billing_multiplier: value.billing_multiplier,
            allowed_model_ids: value.allowed_model_ids,
            allowed_provider_ids: value.allowed_provider_ids,
            allowed_provider_key_ids: value.allowed_provider_key_ids,
            visible_user_group_codes: value.visible_user_group_codes,
            is_active: value.is_active,
            is_system: value.is_system,
            sort_order: value.sort_order,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

fn default_group_limit() -> u64 {
    DEFAULT_GROUP_LIMIT
}
