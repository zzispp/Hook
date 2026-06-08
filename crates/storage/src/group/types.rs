use rust_decimal::Decimal;
use types::model::PatchField;

#[derive(Clone, Debug, PartialEq)]
pub struct BillingGroupRecordInput {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub billing_multiplier: Decimal,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_group_ids: Vec<String>,
    pub allowed_provider_key_group_ids: Vec<String>,
    pub visible_user_group_codes: Vec<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub sort_order: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BillingGroupRecordPatch {
    pub name: Option<String>,
    pub description: PatchField<String>,
    pub billing_multiplier: Option<Decimal>,
    pub allowed_model_ids: PatchField<Vec<String>>,
    pub allowed_provider_group_ids: PatchField<Vec<String>>,
    pub allowed_provider_key_group_ids: PatchField<Vec<String>>,
    pub visible_user_group_codes: PatchField<Vec<String>>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
}
